use clap::Parser;
use glob::glob;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    /// Directory
    directory: Option<PathBuf>,
}
fn main() {
    let cli = Cli::parse();
    let directory = match cli.directory {
        Some(path) => {
            let mut dir = current_dir().unwrap();
            dir.push(path);
            dir
        }
        None => current_dir().unwrap(),
    };
    make_markdown_for_dir(&directory).unwrap();
}

fn make_markdown_for_dir(dir: &Path) -> std::io::Result<()> {
    let dir_contend = read_dir(dir, None)?;
    let name = dir.file_name().unwrap().to_str().unwrap();
    let mut result = format!("# {}\n\n", name);
    for contend in dir_contend {
        if contend.is_dir() && !contend.ends_with(".git") {
            let mut file = contend.clone();
            file.push(contend.file_name().unwrap().to_str().unwrap());
            result.push_str(&link_markdown(dir, &file));
            make_markdown_for_dir(&contend)?;
        } else if match contend.extension() {
            Some(ex) => ex,
            None => continue,
        } == "md"
        {
            result.push_str(&link_markdown(dir, &contend));
        }
    }

    let file_name = format!("{}.md", name);
    create_file(&dir.join(PathBuf::from(file_name)), result)
}

fn link_markdown(root_dir: &Path, file: &Path) -> String {
    let dir = file.strip_prefix(root_dir).unwrap();

    let parent = dir.parent().unwrap_or(&Path::new(""));
    let file = match dir.file_stem() {
        Some(se) => parent.join(se),
        None => dir.to_path_buf(),
    };
    format!(
        "- [{}](./{})\n",
        dir.file_stem().unwrap().to_str().unwrap(),
        file.display()
    )
}

// gives a string with all the files in that match a path pattern
pub fn read_dir(dir: &Path, file_ext: Option<&str>) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let search = match file_ext {
        Some(ext) => dir.join(format!("*{ext}")),
        None => dir.join("*"),
    };

    let dir = match search.to_str() {
        Some(dir) => dir,
        None => return Err(std::io::ErrorKind::NotFound.into()),
    };
    for entry in glob(dir).expect("Failed to read glob pattern") {
        match entry {
            Ok(entry) => result.push(entry),
            Err(err) => return Err(err.into_error()),
        }
    }
    Ok(result)
}

fn create_file(path: &Path, content: String) -> std::io::Result<()> {
    // create a file
    let description_file = File::create(path)?;

    // write to the file
    description_file.write_at(content.as_bytes(), 0)?;

    Ok(())
}
