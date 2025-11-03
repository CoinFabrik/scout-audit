use crate::{scout::core::nightly_runner::set_up_environment, util::print::print_full_error};
use anyhow::{Context, Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
#[cfg(not(windows))]
use std::process::Command;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub fn run_subprocess<I: Serialize, O: DeserializeOwned>(
    toolchain: &str,
    exec: &PathBuf,
    input: &I,
) -> Result<O> {
    let (input_path, output_path) = get_paths(&input)?;

    let env = set_up_environment(toolchain)?;
    do_it(exec, env, &input_path, &output_path)?;

    //Ignore errors when deleting the temporary file.
    let _ = std::fs::remove_file(input_path);

    let output = {
        let mut file =
            File::open(&output_path).with_context(|| "Failed to open output temporary file")?;
        let mut s = String::new();
        file.read_to_string(&mut s)
            .with_context(|| format!("Error while reading {}", output_path))?;

        serde_json::from_str::<O>(&s).map_err(|_| anyhow!("Failed to parse output file"))
    }?;

    //Ignore errors when deleting the temporary file.
    let _ = std::fs::remove_file(output_path);

    Ok(output)
}

fn get_paths<I: Serialize>(input: &I) -> Result<(String, String)> {
    let input_path = {
        let (mut file, path) =
            crate::util::temp_path().with_context(|| "Failed to create temporary file")?;

        let serialized = serde_json::to_string(&input)?;
        let _ = file.write(serialized.as_bytes())?;
        path
    };
    let output_path = crate::util::temp_path()
        .with_context(|| "Failed to create temporary file")?
        .1;

    Ok((input_path, output_path))
}

#[cfg(not(windows))]
fn do_it(
    exec: &PathBuf,
    environment: HashMap<String, String>,
    input_path: &String,
    output_path: &String,
) -> Result<()> {
    if !std::fs::exists(exec)? {
        return Err(anyhow!("Pre-emptive check: path {:?} does not exist", exec));
    }

    let mut command = Command::new(exec);
    command.args([input_path, output_path]);
    crate::util::command::set_env(&mut command, &environment);

    let mut child = command
        .spawn()
        .with_context(|| "Failed to spawn child process with nightly toolchain")?;

    let code = child
        .wait()?
        .code()
        .ok_or_else(|| anyhow!("Failed to get exit code from child process"))?;

    if code != 0 {
        Err(anyhow!("Child process failed"))
    } else {
        Ok(())
    }
}

pub fn subprocess_wrapper<I: DeserializeOwned, O: Serialize, F: FnOnce(&I) -> O>(f: F) {
    let code = match subprocess_wrapper_with_result(f) {
        Ok(_) => 0,
        Err(e) => {
            print_full_error(&e);
            1
        }
    };
    std::process::exit(code);
}

fn subprocess_wrapper_with_result<I: DeserializeOwned, O: Serialize, F: FnOnce(&I) -> O>(
    f: F,
) -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let i = args.first().with_context(|| "Input path missing")?.clone();
    let o = args.get(1).with_context(|| "Output path missing")?.clone();

    let i = {
        let mut file = File::open(&i)?;
        let mut s = String::new();
        file.read_to_string(&mut s)
            .with_context(|| format!("Error while reading {}", i))?;

        serde_json::from_str::<I>(&s).map_err(|_| anyhow!("Failed to parse input data"))?
    };

    let result = f(&i);

    {
        let mut file = File::create(&o).map_err(|_| anyhow!("Failed to open {}", o))?;
        let serialized =
            serde_json::to_string(&result).with_context(|| "Failed to serialize output data")?;
        let _ = file
            .write(serialized.as_bytes())
            .with_context(|| "Failed to write output data")?;
    }

    Ok(())
}
