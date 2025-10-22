use std::{
    ffi::OsStr,
    path::Path,
    process::{Command as StdCommand, Stdio},
};

use anyhow::{ensure, Context, Result};

use super::env;

pub struct Command {
    command: StdCommand,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            command: StdCommand::new(program),
        }
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    #[cfg(windows)]
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.command.env_remove(key);
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.command.stderr(cfg);
        self
    }

    pub fn sanitize_environment(&mut self) -> &mut Self {
        self.env_remove(env::RUSTC);
        self.env_remove(env::RUSTUP_TOOLCHAIN);
        self
    }

    pub fn success(&mut self) -> Result<()> {
        let status = self
            .command
            .status()
            .with_context(|| format!("Could not get status of '{:?}'", self.command))?;

        ensure!(status.success(), "command failed: '{:?}'", self.command);

        Ok(())
    }
}

pub fn set_env(cmd: &mut std::process::Command, environment: &std::collections::HashMap<String, String>){
    for (k, v) in environment.iter(){
        cmd.env(k, v);
    }
}
