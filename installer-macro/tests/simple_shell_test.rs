use installer_macro::shell_installer;

pub struct ShellContext;

#[derive(Debug)]
pub enum Error {
    TestError,
}

impl ShellContext {
    pub fn command(&self, _cmd: &'static str, _args: &[&str]) -> Result<String, Error> {
        Ok("test".to_string())
    }
}

#[shell_installer]
pub fn simple_installer(_ctx: &ShellContext, _args: &[String]) -> Result<(), Error> {
    // This is a minimal installer that should compile
    Ok(())
}

#[test]
fn test_simple_installer_compiles() {
    // Just check that the macro generates valid code
    assert!(SIMPLE_INSTALLER_SHELL.contains("#!/bin/sh"));
    assert!(SIMPLE_INSTALLER_SHELL.contains("set -euf"));
}
