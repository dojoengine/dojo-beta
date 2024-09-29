#![allow(unused)]

use std::str::FromStr;

use syn::spanned::Spanned;

use crate::utils::{parse_bool, parse_int, parse_path, parse_string};

pub const DEFAULT_ERROR_CONFIG: Config = Config::new(false);

pub struct Config {
    pub crate_name: Option<syn::Path>,
    pub kind: Kind,
}

impl Config {
    const fn new(is_test: bool) -> Self {
        Self {
            crate_name: None,
            kind: Kind::Binary(BinaryConfig {
                dev: is_test,
                fee: None,
                db_dir: None,
                log_path: None,
                accounts: None,
                validation: None,
                block_time: None,
                program_path: None,
            }),
        }
    }
}

pub enum Kind {
    Binary(BinaryConfig),
    Embedded(EmbeddedConfig),
}

pub struct BinaryConfig {
    pub dev: bool,
    pub fee: Option<syn::Expr>,
    pub db_dir: Option<syn::Expr>,
    pub log_path: Option<syn::Expr>,
    pub accounts: Option<syn::Expr>,
    pub validation: Option<syn::Expr>,
    pub block_time: Option<syn::Expr>,
    pub program_path: Option<syn::Expr>,
}

pub struct EmbeddedConfig {
    pub dev: bool,
    pub fee: Option<syn::Expr>,
    pub db_dir: Option<syn::Expr>,
    pub accounts: Option<syn::Expr>,
    pub validation: Option<syn::Expr>,
    pub block_time: Option<syn::Expr>,
}

pub struct ConfigBuilder {
    crate_name: Option<syn::Path>,
    flavor: Option<RunnerFlavor>,
    dev: bool,
    accounts: Option<syn::Expr>,
    fee: Option<syn::Expr>,
    validation: Option<syn::Expr>,
    db_dir: Option<syn::Expr>,
    block_time: Option<syn::Expr>,
    log_path: Option<syn::Expr>,
    program_path: Option<syn::Expr>,
}

impl ConfigBuilder {
    fn new(is_test: bool) -> Self {
        Self {
            fee: None,
            db_dir: None,
            flavor: None,
            dev: is_test,
            accounts: None,
            log_path: None,
            validation: None,
            block_time: None,
            crate_name: None,
            program_path: None,
        }
    }

    fn set_crate_name(
        &mut self,
        name: syn::Lit,
        span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if self.crate_name.is_some() {
            return Err(syn::Error::new(span, "`crate` set multiple times."));
        }

        let name_path = parse_path(name, span, "crate")?;
        self.crate_name = Some(name_path);
        Ok(())
    }

    fn set_flavor(&mut self, flavor: syn::Lit, span: proc_macro2::Span) -> Result<(), syn::Error> {
        if self.flavor.is_some() {
            return Err(syn::Error::new(span, "`flavor` set multiple times."));
        }

        let str = parse_string(flavor, span, "`flavor`")?;
        let flavor = RunnerFlavor::from_str(&str).map_err(|err| syn::Error::new(span, err))?;

        self.flavor = Some(flavor);
        Ok(())
    }

    fn set_db_dir(&mut self, db_dir: syn::Expr, span: proc_macro2::Span) -> Result<(), syn::Error> {
        if self.db_dir.is_some() {
            return Err(syn::Error::new(span, "`db_dir` set multiple times."));
        }

        self.db_dir = Some(db_dir);
        Ok(())
    }

    fn set_fee(&mut self, fee: syn::Expr, span: proc_macro2::Span) -> Result<(), syn::Error> {
        if self.fee.is_some() {
            return Err(syn::Error::new(span, "`fee` set multiple times."));
        }

        self.fee = Some(fee);
        Ok(())
    }

    fn set_validation(
        &mut self,
        validation: syn::Expr,
        span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if self.validation.is_some() {
            return Err(syn::Error::new(span, "`validation` set multiple times."));
        }

        self.validation = Some(validation);
        Ok(())
    }

    fn set_block_time(
        &mut self,
        block_time: syn::Expr,
        span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if self.block_time.is_some() {
            return Err(syn::Error::new(span, "`block_time` set multiple times."));
        }

        self.block_time = Some(block_time);
        Ok(())
    }

    fn set_accounts(
        &mut self,
        accounts: syn::Expr,
        span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if self.accounts.is_some() {
            return Err(syn::Error::new(span, "`accounts` set multiple times."));
        }

        self.accounts = Some(accounts);
        Ok(())
    }

    fn set_program_path(
        &mut self,
        program_path: syn::Expr,
        span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if self.program_path.is_some() {
            return Err(syn::Error::new(span, "`program_path` set multiple times."));
        }

        if let Some(flavor) = &self.flavor {
            if flavor != &RunnerFlavor::Binary {
                return Err(syn::Error::new(
                    span,
                    "`program_path` can only be set for runner flavor: `binary`.",
                ));
            }
        }

        self.program_path = Some(program_path);
        Ok(())
    }

    fn build_as_binary(mut self) -> Result<Config, syn::Error> {
        Ok(Config {
            crate_name: self.crate_name,
            kind: Kind::Binary(BinaryConfig {
                dev: self.dev,
                fee: self.fee,
                db_dir: self.db_dir,
                log_path: self.log_path,
                accounts: self.accounts,
                validation: self.validation,
                block_time: self.block_time,
                program_path: self.program_path,
            }),
        })
    }

    fn build_as_embedded(mut self) -> Result<Config, syn::Error> {
        // if self.program_path.is_some() {
        //     return Err(syn::Error::new(
        //         span,
        //         "`program_path` can only be set for runner flavor: `binary`.",
        //     ));
        // }

        Ok(Config {
            crate_name: self.crate_name,
            kind: Kind::Embedded(EmbeddedConfig {
                dev: self.dev,
                fee: self.fee,
                db_dir: self.db_dir,
                accounts: self.accounts,
                validation: self.validation,
                block_time: self.block_time,
            }),
        })
    }

    fn build(mut self) -> Result<Config, syn::Error> {
        match self.flavor {
            None => self.build_as_binary(),
            Some(ref flavor) => match flavor {
                RunnerFlavor::Binary => self.build_as_binary(),
                RunnerFlavor::Embedded => self.build_as_embedded(),
            },
        }
    }
}

#[derive(PartialEq)]
enum RunnerFlavor {
    Embedded,
    Binary,
}

impl FromStr for RunnerFlavor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "embedded" => Ok(RunnerFlavor::Embedded),
            "binary" => Ok(RunnerFlavor::Binary),
            _ => Err(format!(
                "No such runner flavor `{s}`. The runner flavors are `embedded` and `binary`.",
            )),
        }
    }
}

enum RunnerArg {
    Flavor,
    BlockTime,
    Fee,
    Validation,
    Accounts,
    DbDir,
    ProgramPath,
}

impl std::str::FromStr for RunnerArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "block_time" => Ok(RunnerArg::BlockTime),
            "fee" => Ok(RunnerArg::Fee),
            "validation" => Ok(RunnerArg::Validation),
            "accounts" => Ok(RunnerArg::Accounts),
            "db_dir" => Ok(RunnerArg::DbDir),
            "flavor" => Ok(RunnerArg::Flavor),
            "program_path" => Ok(RunnerArg::ProgramPath),
            _ => Err(format!(
                "Unknown attribute {s} is specified; expected one of: `flavor`, `fee`, \
                 `validation`, `accounts`, `db_dir`, `block_time`, `program_path`",
            )),
        }
    }
}

pub fn build_config(
    _input: &crate::item::ItemFn,
    args: crate::entry::AttributeArgs,
    is_test: bool,
) -> Result<Config, syn::Error> {
    let mut config = ConfigBuilder::new(is_test);

    for arg in args {
        match arg {
            syn::Meta::NameValue(namevalue) => {
                let ident = namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string()
                    .to_lowercase();

                // the ident of the attribute
                let ident = ident.as_str();
                let arg = RunnerArg::from_str(ident)
                    .map_err(|err| syn::Error::new_spanned(&namevalue, err))?;

                match arg {
                    RunnerArg::Flavor => {
                        let lit = match &namevalue.value {
                            syn::Expr::Lit(syn::ExprLit { lit, .. }) => lit,
                            expr => return Err(syn::Error::new_spanned(expr, "Must be a literal")),
                        };
                        config.set_flavor(lit.clone(), Spanned::span(&namevalue))?
                    }
                    RunnerArg::BlockTime => {
                        let expr = &namevalue.value;
                        config.set_block_time(expr.clone(), Spanned::span(&namevalue))?
                    }
                    RunnerArg::Validation => {
                        let expr = &namevalue.value;
                        config.set_validation(expr.clone(), Spanned::span(&namevalue))?
                    }
                    RunnerArg::Accounts => {
                        let expr = &namevalue.value;
                        config.set_accounts(expr.clone(), Spanned::span(&namevalue))?;
                    }
                    RunnerArg::DbDir => {
                        let expr = &namevalue.value;
                        config.set_db_dir(expr.clone(), Spanned::span(&namevalue))?
                    }
                    RunnerArg::Fee => {
                        let expr = &namevalue.value;
                        config.set_fee(expr.clone(), Spanned::span(&namevalue))?
                    }
                    RunnerArg::ProgramPath => {
                        let expr = &namevalue.value;
                        config.set_program_path(expr.clone(), Spanned::span(&namevalue))?
                    }
                }
            }

            other => {
                return Err(syn::Error::new_spanned(other, "Unknown attribute inside the macro"));
            }
        }
    }

    Ok(config.build()?)
}
