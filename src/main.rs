use clap::{Parser, Subcommand, ValueEnum};
use libflatpak::{InstalledRef, Remote, Transaction, gio::{Cancellable, prelude::FileExt}, prelude::{InstallationExt, InstallationExtManual, RefExt, RemoteExt, RemoteRefExt, TransactionExt}};
use serde::{Deserialize, Serialize};

use crate::MyFlatpakTransactionOperationType::{Install, InstallBundle, LastType, Uninstall, Update};

fn system_installation() -> libflatpak::Installation {
    libflatpak::Installation::new_system(Cancellable::NONE).unwrap()
}

fn user_installation() -> libflatpak::Installation {
    libflatpak::Installation::new_user(Cancellable::NONE).unwrap()
}


fn installed_refs_names(refs : Vec<InstalledRef>) -> Vec<String> {
    refs.iter().map(
        |installed_ref| {
            installed_ref.format_ref().unwrap_or_default().to_string()
        }
    ).collect()
}

static HELP_MESSAGE : &str = 
"
Commands:
    install         <user/system> <remote> <reference>    -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    remove          <user/system> <reference>             -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    upgrade         <user/system> <reference>             -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    update-cache    <user/system> <remote>
    list-installed  <user/system>                         -> List<reference : String>
    list-upgradable <user/system>                         -> List<reference : String>
    appstream-path  <user/system> <remote>                -> path : String
    add-remote      <user/system> <name> <url>               !!UNTESTED!!
    remove-remote   <user/system> <name>                     !!UNTESTED!!
    remotes         <user/system>                         -> List<FlatpakRemote>
    info            <user/system> <remote> <reference>    -> MyFlatpakRemoteRefInfo
";

trait MyTransactionExt {
    fn report_operations(&self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyFlatpakTransactionOperation {
    reference : String,
    operation_type : MyFlatpakTransactionOperationType,
    download_size : u64,
    installed_size : u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MyFlatpakTransactionOperationType {
    Install,Update,InstallBundle,Uninstall,LastType
}

impl MyFlatpakTransactionOperationType {
    fn from(other : libflatpak::TransactionOperationType) -> MyFlatpakTransactionOperationType {
        match other {
            libflatpak::TransactionOperationType::Install => Install,
            libflatpak::TransactionOperationType::Update => Update,
            libflatpak::TransactionOperationType::InstallBundle => InstallBundle,
            libflatpak::TransactionOperationType::Uninstall => Uninstall,
            libflatpak::TransactionOperationType::LastType => LastType,
            _ => todo!(),
        }
    }
}

impl MyTransactionExt for Transaction {
    fn report_operations(&self) {
        let operations : Vec<MyFlatpakTransactionOperation> = self.operations().iter().map(
            |operation| {
                MyFlatpakTransactionOperation {
                    reference: operation.get_ref().unwrap_or_default().to_string(),
                    operation_type: MyFlatpakTransactionOperationType::from(operation.operation_type()),
                    download_size: operation.download_size(),
                    installed_size: operation.installed_size(),
                }
            }
        ).collect();
        println!("{}\0",serde_json::to_string(&operations).expect("Couldn't serialize operations : Vec<MyFlatpakTransactionOperation>"));
        self.connect_new_operation(
            |_transaction , _operation , progress  | {
                progress.connect_changed(
                    |progress| {
                        print!("{}\0",progress.progress());                    
                    }
                );   
            }
        );
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Install {installation,remote,reference} => {
            let transaction = Transaction::for_installation(&installation.get(), Cancellable::NONE).expect("Could not initiate transaction");
            transaction.add_install(&remote, &reference, &[]).expect("install couldn't be added to transaction");
            transaction.report_operations();
            transaction.run(Cancellable::NONE).expect("transaction could not be run");
        }

        Command::Remove {installation, reference} => {
            let transaction = Transaction::for_installation(&installation.get(), Cancellable::NONE).expect("Could not initiate transaction");
            transaction.add_uninstall(&reference).expect("remove couldn't be added to transaction");
            transaction.report_operations();
            transaction.run(Cancellable::NONE).expect("transaction could not be run");
        }

        Command::Upgrade {installation,reference} => {
            let transaction = Transaction::for_installation(&installation.get(), Cancellable::NONE).expect("Could not initiate transaction");
            transaction.add_update(&reference , &[] , None).expect("upgrade couldn't be added to transaction");
            transaction.report_operations();
            transaction.run(Cancellable::NONE).expect("transaction could not be run");
        }

        Command::UpdateCache {installation,remote} => {
            installation.get().update_appstream_full_sync(&remote, None, None, Cancellable::NONE).expect("Couldn't update cache");
        }

        Command::ListInstalled { installation } => {
            let refs = installed_refs_names(installation.get().list_installed_refs(Cancellable::NONE).expect("Couldn't list installed flatpaks"));
            println!("{}",serde_json::to_string(&refs).expect("Unable to serialize refs : Vec<String>"))
        }

        Command::ListUpgradable { installation } => {
            let refs = installed_refs_names(installation.get().list_installed_refs_for_update(Cancellable::NONE).expect("Couldn't list installed flatpaks")); 
            println!("{}",serde_json::to_string(&refs).expect("Unable to serialize refs : Vec<String>"))            
        }

        Command::AppstreamPath {installation , remote } => {
            let remote = installation.get().remote_by_name(&remote, Cancellable::NONE).expect("No remote found with that name");
            let app_stream_path = remote
                .appstream_dir(None)
                .and_then(|file| file.path())
                .and_then(|path| path.to_str().map(str::to_owned));

            println!(
                "{}",
                serde_json::to_string(&app_stream_path).unwrap()
            );
        }

        Command::AddRemote {installation , name, url } => {
            let remote = Remote::new(&name);
            remote.set_url(&url);
            installation.get().add_remote(&remote, false, Cancellable::NONE).expect("Could not add remote.");
        }

        Command::RemoveRemote {installation , name } => {
            installation.get().remove_remote(&name, Cancellable::NONE).expect("Couldn't remove remote.");
        }

        Command::Remotes {installation} => {
            let remotes : Vec<FlatpakRemote> = 
                    installation.get().list_remotes(Cancellable::NONE).expect("Remotes couldn't be retreived").iter().map(
                        | remote | {
                            FlatpakRemote {
                                name: remote.name().unwrap_or_default().to_string(),
                                url: remote.url().unwrap_or_default().to_string(),
                            }
                        }
                    ).collect();

            println!("{}",serde_json::to_string(&remotes).expect("Unable to serialize remotes : Vec<FlatpakRemote>"));
        }
        Command::Info { installation, remote, reference : reference_name } => {
            let installation = installation.get();
            let reference = libflatpak::Ref::parse(&reference_name).expect("Couldn't find reference");
            let remote_ref = 
                installation.fetch_remote_ref_sync(
                    &remote, 
                    reference.kind(), 
                    &reference.name().unwrap_or_default().to_string(),
                    Some(&reference.arch().unwrap_or_default().to_string()), 
                    Some(&reference.branch().unwrap_or_default().to_string()), 
                    Cancellable::NONE
                ).expect("Remote ref not found");
            let info = MyFlatpakRemoteRefInfo {
                reference: reference_name,
                download_size: remote_ref.download_size(),
                installed_size: remote_ref.installed_size(),
            };
            println!("{}",serde_json::to_string(&info).expect("Couldn't serialized info : MyFlatpakRemoteRefInfo"));
        },
        Command::EndPoints => {
            println!("{}",HELP_MESSAGE);
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlatpakRemote {
    name : String,
    url : String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyFlatpakRemoteRefInfo {
    reference : String,
    download_size : u64,
    installed_size : u64,
}

#[derive(Parser, Debug)]
#[command(name = "flatpak-cli")]
#[command(version)]
#[command(about = "Flatpak backend CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Install {
        #[arg(value_enum)]
        installation: Installation,
        remote: String,
        reference: String,
    },

    Remove {
        #[arg(value_enum)]
        installation: Installation,
        reference: String,
    },

    Upgrade {
        #[arg(value_enum)]
        installation: Installation,

        reference: String,
    },

    UpdateCache {
        #[arg(value_enum)]
        installation: Installation,
        remote: String,
    },

    ListInstalled {
        #[arg(value_enum)]
        installation: Installation,
    },

    ListUpgradable {
        #[arg(value_enum)]
        installation: Installation,
    },

    AppstreamPath {
        #[arg(value_enum)]
        installation: Installation,
        remote: String,
    },

    AddRemote {
        #[arg(value_enum)]
        installation: Installation,
        name: String,
        url: String,
    },

    RemoveRemote {
        #[arg(value_enum)]
        installation: Installation,
        name: String,
    },

    Remotes {
        #[arg(value_enum)]
        installation: Installation,
    },
    Info {
        #[arg(value_enum)]
        installation: Installation,
        remote : String,
        reference: String,
    },
    EndPoints,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Installation {
    User,
    System,
}

impl Installation {
    fn get(&self) -> libflatpak::Installation {
        match self {
            Installation::User => user_installation(),
            Installation::System => system_installation(),
        }
    }
}

