use std::env;
use std::fs;
use std::process;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Resume {
    name: String,
    title: String,
    location: String,
    email: String,
    #[serde(default)]
    experience: Vec<Experience>,
}

#[derive(Debug, Deserialize)]
struct Experience {
    company: String,
    role: String,
    start: String,
    end: String,
    summary: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!(" resume-cli greet <name>");
        eprintln!(" resume-cli show <path-to-resume.toml>");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "greet" => {
            if args.len() < 3 {
                eprintln!("Usage: resume-cli great <name>");
                process::exit(1);
            }
            let name = &args[2];
            println!("Hello, {name}! resume-cli at your service.");
        }

        "show" => {
            if args.len() < 3 {
                eprintln!("Usage: resume-cli show <path-to-resume.toml>");
                process::exit(1);
            }
            let path = &args[2];
            let resume = load_resume(path);
            print_resume(&resume);
        }

        "build" => {
            if args.len() < 4 {
                eprintln!("Usage: resume-cli build <path-to-resume.toml> <output.pdf>");
                process::exit(1);
            }
            let input = &args[2];
            let output = &args[3];

            let resume = load_resume(input);
            build_pdf(&resume, output).unwrap_or_else(|err| {
                eprintln!("Failed to build PDF: {err}");
                process::exit(1)
            });

            println!("Wrote PDF to {output}");
        }

        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

fn load_resume(path: &str) -> Resume {
    let contents = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("Failed to read {path}: {err}");
        process::exit(1);
    });

    toml::from_str(&contents).unwrap_or_else(|err| {
        eprintln!("Failed to parse TOML in {path}: {err}");
        process::exit(1);
    })
}

fn print_resume(resume: &Resume) {
    println!("Name:     {}", resume.name);
    println!("Title:    {}", resume.title);
    println!("Location: {}", resume.location);
    println!("Email:    {}", resume.email);
    println!();
    println!("Experience:");
    for exp in &resume.experience {
        println!(
            "- {} - {} ({} -> {})",
            exp.company, exp.role, exp.start, exp.end
        );
        println!("  {}", exp.summary);
    }
}

fn build_pdf(resume: &Resume, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use genpdf::{Element, elements, fonts, style};

    // Load font family from ./fonts/*.ttf
    let font_family =
        fonts::from_files("./fonts", "Inter_24pt", None).expect("Failed to load font family");

    let mut doc = genpdf::Document::new(font_family);

    doc.set_title(format!("{} – {}", resume.name, resume.title));

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let mut layout = elements::LinearLayout::vertical();

    // Header: name
    let name_style = style::Style::new().bold().with_font_size(20);
    layout.push(
        elements::Paragraph::new(&resume.name)
            .aligned(genpdf::Alignment::Center)
            .styled(name_style),
    );

    // Title
    let title_style = style::Style::new().with_font_size(14);
    layout.push(
        elements::Paragraph::new(&resume.title)
            .aligned(genpdf::Alignment::Center)
            .styled(title_style),
    );

    // Location + email
    layout.push(
        elements::Paragraph::new(format!("{} · {}", resume.location, resume.email))
            .aligned(genpdf::Alignment::Center),
    );

    // Spacer
    layout.push(elements::Break::new(1));

    // Experience section
    if !resume.experience.is_empty() {
        let section_title_style = style::Style::new().bold().with_font_size(14);
        layout.push(elements::Paragraph::new("Experience").styled(section_title_style));
        layout.push(elements::Break::new(0.5));

        for exp in &resume.experience {
            let exp_header_style = style::Style::new().bold();
            layout.push(
                elements::Paragraph::new(format!(
                    "{} — {} ({} – {})",
                    exp.company, exp.role, exp.start, exp.end
                ))
                .styled(exp_header_style),
            );
            layout.push(elements::Paragraph::new(&exp.summary));
            layout.push(elements::Break::new(0.5));
        }
    }

    doc.push(layout);
    doc.render_to_file(output_path)?;
    Ok(())
}
