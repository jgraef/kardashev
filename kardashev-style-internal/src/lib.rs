mod manifest;
mod rename;

use std::{
    collections::HashMap,
    fmt::Debug,
    io::Write,
    path::{
        Path,
        PathBuf,
    },
    sync::Mutex,
};

use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{
        ParserOptions,
        StyleSheet,
    },
    visitor::Visit,
};

use crate::{
    manifest::StyleMetadata,
    rename::RenameClassNames,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while parsing input: {path}")]
    Grass {
        #[source]
        source: Box<grass::Error>,
        path: PathBuf,
    },
    #[error("Error while parsing CSS for transformation: {path}")]
    LightningCssParse { message: String, path: PathBuf },
    #[error("Error while printing CSS for transformation: {path}")]
    LightningCssPrint {
        #[source]
        source: lightningcss::error::PrinterError,
        path: PathBuf,
    },
    #[error("Error while creating output directory: {path}")]
    CreateDirectory {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Error while writing output CSS: {path}")]
    WriteOutput {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("Crate name could not be determined.")]
    NoCrateName {
        #[source]
        source: std::env::VarError,
    },
    #[error("Output directory could not be determined.")]
    NoOutputPath {
        #[source]
        source: std::env::VarError,
    },
    #[error("Manifest directory could not be determined.")]
    NoManifestDir {
        #[source]
        source: std::env::VarError,
    },
    #[error("Could not read package manifest: {path}")]
    ReadManifest {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Could not parse package manifest: {path}\n{source}")]
    ParseManifest {
        #[source]
        source: toml::de::Error,
        path: PathBuf,
    },
}

#[derive(Debug)]
pub struct Output {
    pub class_names: HashMap<String, String>,
    pub css: String,
    pub css_path: PathBuf,
}

pub fn prepare_import(input_path: &Path, track: impl FnMut(&Path)) -> Result<Output, Error> {
    let track_fs = TrackFs {
        track: Mutex::new(track),
    };
    let options = grass::Options::default()
        .fs(&track_fs)
        .style(grass::OutputStyle::Expanded)
        .input_syntax(grass::InputSyntax::Scss);

    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").map_err(|source| Error::NoManifestDir { source })?,
    );

    let manifest_path = manifest_dir.join("Cargo.toml");
    let metadata = StyleMetadata::read(&manifest_path)?;

    let crate_name = if let Some(crate_name) = metadata.crate_name {
        crate_name
    }
    else {
        std::env::var("CARGO_CRATE_NAME").map_err(|source| Error::NoCrateName { source })?
    };

    let input_path = manifest_dir.join(input_path);
    if !input_path.exists() {
        return Err(Error::FileNotFound {
            path: input_path.to_owned(),
        });
    }

    let mut class_names = HashMap::new();

    let hash = fasthash::xx::hash64(input_path.as_os_str().as_encoded_bytes());
    let file_id = bs58::encode(&hash.to_be_bytes()).into_string();

    let mut visitor = RenameClassNames {
        class_names: &mut class_names,
        file_id: &file_id,
        crate_name: &crate_name,
    };

    let css = grass::from_path(&input_path, &options).map_err(|source| {
        Error::Grass {
            source,
            path: input_path.to_owned(),
        }
    })?;

    let parser_options = ParserOptions::default();
    let mut css = StyleSheet::parse(&css, parser_options).map_err(|source| {
        Error::LightningCssParse {
            message: source.to_string(),
            path: input_path.to_owned(),
        }
    })?;

    css.visit(&mut visitor).expect("visitor can't fail");

    let printer_options = PrinterOptions::default();
    let output = css.to_css(printer_options).map_err(|source| {
        Error::LightningCssPrint {
            source,
            path: input_path.to_owned(),
        }
    })?;

    let output_path = metadata
        .output
        .as_deref()
        .unwrap_or_else(|| Path::new("./target/css/kardashev-ui"));
    std::fs::create_dir_all(output_path).map_err(|source| {
        Error::CreateDirectory {
            source,
            path: output_path.to_owned(),
        }
    })?;
    let output_path = output_path.join(format!("{crate_name}-{file_id}.scss"));

    let mut output_css = vec![];
    write!(
        &mut output_css,
        r#"
/*
    Crate: {crate_name}
    Input: {}
    File ID: {file_id}
    Class Names:
"#,
        input_path.display()
    )
    .unwrap();
    for (original_class_name, mangled_class_name) in &class_names {
        writeln!(
            &mut output_css,
            "        {original_class_name} -> {mangled_class_name}"
        )
        .unwrap()
    }
    writeln!(&mut output_css, "*/\n\n{}\n", output.code).unwrap();
    std::fs::write(&output_path, &output_css).map_err(|source| {
        Error::WriteOutput {
            source,
            path: output_path.clone(),
        }
    })?;

    Ok(Output {
        class_names,
        css: String::from_utf8(output_css).expect("output css contains invalid UTF-8"),
        css_path: output_path,
    })
}

struct TrackFs<F> {
    track: Mutex<F>,
}

impl<F> Debug for TrackFs<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackFs").finish_non_exhaustive()
    }
}

impl<F: FnMut(&Path)> grass::Fs for TrackFs<F> {
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        let mut track = self.track.lock().expect("mutex poisoned");
        track(path);
        std::fs::read(path)
    }
}
