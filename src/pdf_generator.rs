// src/pdf_generator.rs

use std::process::Stdio;
use tokio::fs;
use tokio::process::Command as TokioCommand;

pub async fn generate_pdf(
    header: &str,
    body: &str,
    output_filename: &str,
) -> Result<(), String> {
    // Assemble the full LaTeX code
    let latex_code = format!(
        r#"{header}
\begin{{document}}
{body}
\end{{document}}
"#,
        header = header,
        body = body
    );

    // Write LaTeX code to a temporary file
    let latex_file_path = format!("{}.tex", output_filename);
    fs::write(&latex_file_path, latex_code)
        .await
        .map_err(|e| format!("Failed to write LaTeX file: {}", e))?;

    // Run pdflatex to generate PDF
    let pdflatex_status = TokioCommand::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(&latex_file_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("Failed to run pdflatex: {}", e))?;

    if !pdflatex_status.success() {
        return Err("pdflatex failed to compile the LaTeX code.".into());
    }

    // Open the PDF with the system's default PDF viewer
    let pdf_file = format!("{}.pdf", output_filename);

    open::that(pdf_file).map_err(|e| format!("Failed to open PDF: {}", e))?;

    // Cleanup auxiliary files (optional)
    let _ = fs::remove_file(format!("{}.aux", output_filename)).await;
    let _ = fs::remove_file(format!("{}.log", output_filename)).await;
    // Optionally remove the .tex file
    // let _ = fs::remove_file(&latex_file_path).await;

    Ok(())
}
