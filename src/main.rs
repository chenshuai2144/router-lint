use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug)]
struct ReadFileError(String);

fn main() {
    // let args = Cli::from_args();
    // let path = &args.path;
    // // display 可以转化成需要显示的文案
    // let path_str: String = path.as_path().display().to_string();

    // let content = std::fs::read_to_string(&args.path)
    //     .map_err(|err| ReadFileError(format!("读取文件异常： `{}`: {}", path_str, err)))?;
    // println!("file content: {}", content);

    // Ok(())

    let bar = ProgressBar::new(1000);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );
    let mut path_str: String = String::from("");
    for _i in 0..100 {
        let args = Cli::from_args();
        path_str = args.path.as_path().display().to_string();
        bar.inc(1);
    }
    bar.println(format!("[+] finished #{}", path_str));
    bar.finish_with_message("done");
    std::process::exit(exitcode::OK);
}
