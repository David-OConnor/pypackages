use crate::util;
use std::{env, fs, path::PathBuf, process::Command};

// https://packaging.python.org/tutorials/packaging-projects/

/// Creates a temporary file which imitates setup.py
fn create_dummy_setup(cfg: &crate::Config, filename: &str) {
    let classifiers = ""; // todo temp
                          // todo add to this
    let version = match cfg.version {
        Some(v) => v.to_string(),
        None => "".into(),
    };

    let cfg = cfg.clone();

    let data = format!(
        r#"import setuptools
 
with open("{}", "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="{}",
    version="{}",
    author="{}",
    author_email="{}",
    description="{}",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="{}",
    packages=setuptools.find_packages(),
    classifiers=[{}],
)
"#,
        cfg.readme_filename.unwrap_or_else(|| "README.md".into()),
        cfg.name.unwrap_or_else(|| "".into()),
        version,
        cfg.author.unwrap_or_else(|| "".into()),
        cfg.author_email.unwrap_or_else(|| "".into()),
        cfg.description.unwrap_or_else(|| "".into()),
        cfg.repo_url.unwrap_or_else(|| "".into()),
        classifiers,
    );

    fs::write(filename, data).expect("Problem writing dummy setup.py");
    if util::wait_for_dirs(&[env::current_dir()
        .expect("Problem finding current dir")
        .join("setup.py")])
    .is_err()
    {
        util::abort("Problem waiting for setup.py to be created.")
    };
}

pub(crate) fn build(bin_path: &PathBuf, lib_path: &PathBuf, cfg: &crate::Config) {
    // todo: Check if they exist; only install if they don't.
    let dummy_setup_fname = "setup_temp_pypackage.py";

    println!("Installing build tools...");
    Command::new("./python")
        .current_dir(bin_path)
        .args(&[
            "-m",
            "pip",
            "install",
            //            "--upgrade",
            "setuptools",
            "twine",
            "wheel",
        ])
        .status()
        .expect("Problem installing build tools");

    create_dummy_setup(cfg, dummy_setup_fname);

    util::set_pythonpath(lib_path);
    println!("Building the package...");
    Command::new(format!("{}/{}", bin_path.to_str().unwrap(), "python"))
        .args(&[dummy_setup_fname, "sdist", "bdist_wheel"])
        .status()
        .expect("Problem building");
    println!("Build complete.");

    if fs::remove_file(dummy_setup_fname).is_err() {
        println!("Problem removing temporary setup file while building ")
    };
}

pub(crate) fn publish(bin_path: &PathBuf, cfg: &crate::Config) {
    let repo_url = cfg
        .package_url
        .clone()
        .unwrap_or("https://test.pypi.org".to_string());

    println!("Uploading to {}", repo_url);
    Command::new(format!("{}/{}", bin_path.to_str().unwrap(), "twine"))
        .args(&[
            //            "-m",
            //            "twine upload",
            "upload",
            &format!("--repository-url {}/", repo_url),
            "dist/*",
        ])
        .status()
        .expect("Problem publishing");
}
