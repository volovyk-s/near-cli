use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    send_to: Option<super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub send_to: super::receiver::SendTo,
}

impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let send_to: super::receiver::SendTo = match item.send_to {
            Some(cli_send_to) => super::receiver::SendTo::from(
                cli_send_to,
                connection_config,
                sender_account_id.clone(),
            )?,
            None => super::receiver::SendTo::send_to(connection_config, sender_account_id.clone())?,
        };
        Ok(Self {
            sender_account_id,
            send_to,
        })
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the account ID of the sender?")
            .interact_text()
            .unwrap()
    }

    fn get_console_command(&self, console_command: &String) -> String {
        format!("{} {} ", console_command, self.sender_account_id)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        console_command: String,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        let console_command: String = self.get_console_command(&console_command);
        self.send_to
            .process(
                unsigned_transaction,
                network_connection_config,
                console_command,
            )
            .await
    }
}
