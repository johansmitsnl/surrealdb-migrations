use anyhow::{anyhow, Result};
use apply::ApplyArgs;
use clap::Parser;
use cli::{Action, Args, CreateAction, ScaffoldAction};
use create::{CreateArgs, CreateEventArgs, CreateMigrationArgs, CreateOperation, CreateSchemaArgs};
use input::SurrealdbConfiguration;

mod apply;
mod cli;
mod config;
mod constants;
mod create;
mod definitions;
mod input;
mod io;
mod list;
mod models;
mod remove;
mod scaffold;
mod surrealdb;
mod validate_version_order;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Action::Scaffold { command } => match command {
            ScaffoldAction::Template { template } => scaffold::template::main(template),
            ScaffoldAction::Schema {
                schema,
                db_type,
                preserve_casing,
            } => scaffold::schema::main(schema, db_type, preserve_casing),
        },
        Action::Create {
            command,
            name,
            down,
            content,
        } => match name {
            Some(name) => {
                let operation = CreateOperation::Migration(CreateMigrationArgs { down, content });
                let args = CreateArgs { name, operation };
                create::main(args)
            }
            None => match command {
                Some(CreateAction::Schema {
                    name,
                    fields,
                    dry_run,
                    schemafull,
                }) => {
                    let operation = CreateOperation::Schema(CreateSchemaArgs {
                        fields,
                        dry_run,
                        schemafull,
                    });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                Some(CreateAction::Event {
                    name,
                    fields,
                    dry_run,
                    schemafull,
                }) => {
                    let operation = CreateOperation::Event(CreateEventArgs {
                        fields,
                        dry_run,
                        schemafull,
                    });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                Some(CreateAction::Migration {
                    name,
                    down,
                    content,
                }) => {
                    let operation =
                        CreateOperation::Migration(CreateMigrationArgs { down, content });
                    let args = CreateArgs { name, operation };
                    create::main(args)
                }
                None => Err(anyhow!("No action specified for `create` command")),
            },
        },
        Action::Remove {} => remove::main(),
        Action::Apply {
            up,
            down,
            address,
            url,
            ns,
            db,
            username,
            password,
            dry_run,
            validate_version_order,
        } => {
            let operation = match (up, down) {
                (Some(_), Some(_)) => {
                    return Err(anyhow!(
                        "You can't specify both `up` and `down` parameters at the same time"
                    ))
                }
                (Some(up), None) => apply::ApplyOperation::UpTo(up),
                (None, Some(down)) => apply::ApplyOperation::Down(down),
                (None, None) => apply::ApplyOperation::Up,
            };
            let db_configuration = SurrealdbConfiguration {
                address,
                url,
                ns,
                db,
                username,
                password,
            };
            let db = surrealdb::create_surrealdb_client(&db_configuration).await?;
            let args = ApplyArgs {
                operation,
                db: &db,
                dir: None,
                display_logs: true,
                dry_run,
                validate_version_order,
            };
            apply::main(args).await
        }
        Action::List {
            address,
            url,
            ns,
            db,
            username,
            password,
            no_color,
        } => {
            let db_configuration = SurrealdbConfiguration {
                address,
                url,
                ns,
                db,
                username,
                password,
            };
            list::main(&db_configuration, no_color).await
        }
    }
}
