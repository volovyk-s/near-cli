use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod change_method;
mod view_method;

/// выбор метода для выполнения
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOptionMethod {
    #[clap(subcommand)]
    method: Option<CliMethod>,
}

#[derive(Debug, Clone)]
pub struct OptionMethod {
    method: Method,
}

impl OptionMethod {
    pub fn from(item: CliOptionMethod) -> color_eyre::eyre::Result<Self> {
        let method = match item.method {
            Some(cli_method) => Method::from(cli_method)?,
            None => Method::choose_method()?,
        };
        Ok(Self { method })
    }
}

impl OptionMethod {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.method.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, clap::Clap)]
enum CliMethod {
    /// Specify a change method
    ChangeMethod(self::change_method::operation_mode::CliOperationMode),
    /// Specify a view method
    ViewMethod(self::view_method::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
enum Method {
    #[strum_discriminants(strum(message = "Change a method"))]
    ChangeMethod(self::change_method::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a method"))]
    ViewMethod(self::view_method::operation_mode::OperationMode),
}

impl Method {
    fn from(item: CliMethod) -> color_eyre::eyre::Result<Self> {
        match item {
            CliMethod::ChangeMethod(cli_operation_mode) => Ok(Method::ChangeMethod(
                self::change_method::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
            CliMethod::ViewMethod(cli_operation_mode) => Ok(Method::ViewMethod(
                self::view_method::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
        }
    }
}

impl Method {
    fn choose_method() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = MethodDiscriminants::iter().collect::<Vec<_>>();
        let methods = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_method = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your method")
            .items(&methods)
            .default(0)
            .interact()
            .unwrap();
        let cli_method = match variants[selected_method] {
            MethodDiscriminants::ChangeMethod => CliMethod::ChangeMethod(Default::default()),
            MethodDiscriminants::ViewMethod => CliMethod::ViewMethod(Default::default()),
        };
        Self::from(cli_method)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::ChangeMethod(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Self::ViewMethod(operation_mode) => operation_mode.process().await,
        }
    }
}
