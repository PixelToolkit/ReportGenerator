use chrono::Local;
use std::{
    error::Error,
    fs::{create_dir, read_dir, remove_file, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::{exit, Command},
};

mod args;

// TODO: templates for default finding (+evidence), common vulns, default section

/*
   report
   - metadata.typ
   - sections
   - - 1.summary.typ
   - - 2.scope.typ
   - - 3.methodology.typ
   - - 4.section.typ
   - findings
   - - 1.finding.typ
*/

const DEFAULT_REPORT_FILE: &str = "report.pdf";
const TMP_FILE: &str = "tmp.typ";
const REPORT_TEMPLATE: &str = include_str!("../others/template.typ");

const EXAMPLE_METADATA: &str = "title:Example Pentest Report
prepared_for:Example prepared for
prepared_by:Example prepared by";

const EXAMPLE_SECTION: &str = "= Example section
Look at this gorgeus sections content
#lorem(200)";

const EXAMPLE_FINDING: &str = "= Example finding
Look at this amazing finding
#lorem(200)";

const EXAMPLE_SUMMARY: &str = "= Summary
Example summary content
#lorem(200)";

const EXAMPLE_METHODOLOGY: &str = "= Methodology
Example methodology
#lorem(200)";

const EXAMPLE_SCOPE: &str = "= Scope
Example scope
#lorem(200)";

fn compile_to_file(report: &str, output: &Option<String>) -> Result<(), Box<dyn Error>> {
    // Write report to temporary file
    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(TMP_FILE)
        .expect("Failed to open temporary file");
    tmp_file.write_all(report.as_bytes())?;

    // Close file
    drop(tmp_file);

    let report_output_file = if let Some(file_name) = output {
        file_name
    } else {
        DEFAULT_REPORT_FILE
    };

    // Use typst to compile the file
    Command::new("typst")
        .args(["compile", TMP_FILE, report_output_file])
        .spawn()
        .expect("Failed to execute typst")
        .wait()
        .expect("Failed to wait for typst");

    // Remove the temporary file
    remove_file(TMP_FILE).expect("Failed to remove temporary file");

    Ok(())
}

fn get_current_date() -> String {
    let date = Local::now();
    date.format("%B %d, %Y").to_string()
}

fn compile_report(
    report_dir: Option<PathBuf>,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // Ensure user provided the report path
    let report_path = report_dir.unwrap_or_else(|| {
        eprintln!("ERROR: Report path not provided");
        exit(1);
    });

    // If directory doesn't exist, error out
    if !report_path.exists() {
        eprintln!("ERROR: Directory doesn't exist");
        exit(1);
    }

    let mut report_title = "[REPORT TITLE - CHANGE ME]";
    let mut prepared_for = "[PREPARED FOR - CHANGE ME]";
    let mut prepared_by = "[PREPARED BY - CHANGE ME]";

    let mut metadata = String::new();
    File::open(report_path.join("metadata.typ"))?.read_to_string(&mut metadata)?;

    // Handle metadata file
    for line in metadata.lines() {
        let split: Vec<&str> = line.split(':').collect();
        if split.len() < 2 {
            continue;
        }
        match split[0] {
            "title" => report_title = split[1],
            "prepared_for" => prepared_for = split[1],
            "prepared_by" => prepared_by = split[1],
            _ => (),
        }
    }

    // Handle sections
    let mut sections = vec![String::new(); read_dir(report_path.join("sections"))?.count()];
    for section in read_dir(report_path.join("sections"))? {
        let section = section?;
        let mut content = String::new();
        File::open(section.path())?.read_to_string(&mut content)?;
        let id = section
            .file_name()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .parse::<usize>()?;
        sections[id - 1] = format!("\n#pagebreak()\n{content}");
    }

    // Handle findings
    let mut findings = vec![String::new(); read_dir(report_path.join("findings"))?.count()];
    for finding in read_dir(report_path.join("findings"))? {
        let finding = finding?;
        let mut content = String::new();
        File::open(finding.path())?.read_to_string(&mut content)?;
        let id = finding
            .file_name()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .parse::<usize>()?;
        findings[id - 1] = format!("\n#pagebreak()\n{content}");
    }

    let sections = sections.join("\n");
    let findings = findings.join("\n");
    let current_date = get_current_date();

    let mut report = REPORT_TEMPLATE.to_owned();
    let context: Vec<(&str, &str)> = vec![
        ("report_title", report_title),
        ("date", &current_date),
        ("prepared_for", prepared_for),
        ("prepared_by", prepared_by),
        ("sections", &sections),
        ("findings", &findings),
    ];
    for element in context {
        report = report.replace(&format!("{{{{ {} }}}}", element.0), element.1);
    }

    compile_to_file(&report, &output)?;

    Ok(())
}

fn new_report(report_dir: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    // Ensure user provided the report path
    let report_path = report_dir.unwrap_or_else(|| {
        eprintln!("ERROR: Report path not provided");
        exit(1);
    });

    // If directory not empty, error out
    if report_path.exists() {
        eprintln!("ERROR: Directory already exists");
        exit(1);
    }

    // Create the file structure
    create_dir(&report_path)?;

    File::create_new(report_path.join("metadata.typ"))?.write_all(EXAMPLE_METADATA.as_bytes())?;

    create_dir(report_path.join("sections"))?;

    File::create_new(report_path.join("sections").join("1.summary.typ"))?
        .write_all(EXAMPLE_SUMMARY.as_bytes())?;
    File::create_new(report_path.join("sections").join("2.scope.typ"))?
        .write_all(EXAMPLE_SCOPE.as_bytes())?;
    File::create_new(report_path.join("sections").join("3.methodology.typ"))?
        .write_all(EXAMPLE_METHODOLOGY.as_bytes())?;
    File::create_new(report_path.join("sections").join("4.example_section.typ"))?
        .write_all(EXAMPLE_SECTION.as_bytes())?;

    create_dir(report_path.join("findings"))?;

    File::create_new(report_path.join("findings").join("1.example_finding.typ"))?
        .write_all(EXAMPLE_FINDING.as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::get_args();
    // println!("{args:?}");

    if let Some(command) = args.subcommand {
        match command.as_ref() {
            // TODO: new finding command (name + optional template)
            // TODO: new section command (name + optional template)
            "new" => {
                new_report(args.dir)?;
            }
            "compile" => {
                compile_report(args.dir, args.output)?;
            }
            _ => {
                eprintln!("Incorrect subcommand. Check --help");
                exit(1);
            }
        }
    } else {
        // GUI
        todo!("GUI");
    }

    Ok(())
}
