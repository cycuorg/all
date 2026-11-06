
use anyhow::{anyhow, Result};
use std::{
    fs,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    assets::{
        CSS, HIGHLIGHT_CSS, HIGHLIGHT_JS, JS, LANDING, LIVERELOAD_JS,
        CLIPBOARD_JS, FONT_AWESOME, FONT_AWESOME_EOT, FONT_AWESOME_EOT_900,
        FONT_AWESOME_EOT_BRANDS, FONT_AWESOME_SVG, FONT_AWESOME_SVG_900,
        FONT_AWESOME_SVG_BRANDS, FONT_AWESOME_TTF, FONT_AWESOME_TTF_900,
        FONT_AWESOME_TTF_BRANDS, FONT_AWESOME_WOFF, FONT_AWESOME_WOFF2,
        FONT_AWESOME_WOFF2_900, FONT_AWESOME_WOFF2_BRANDS,
        FONT_AWESOME_WOFF_900, FONT_AWESOME_WOFF_BRANDS,
    },
    config::UnveilConfig,
    helper,
    html::HtmlBuilder,
    server::Server,
};

pub struct UnveilProject {
    pub root: PathBuf,
    pub markdown: Vec<String>,
    pub livereload: bool,
}

impl Default for UnveilProject {
    fn default() -> Self {
        UnveilProject {
            // TODO: handle this correctly and use it!
            root: PathBuf::from("."),
            markdown: vec![],
            livereload: true,
        }
    }
}

impl UnveilProject {
    /// Read Markdown slides as strings.
    ///
    /// The source files remain in the project's slides directory.
    fn get_markdown_from_file() -> Result<Vec<String>> {
        let mut markdown_contents = vec![];
        let config = UnveilConfig::from_disk("unveil.toml")?;

        // Read slide names from config.
        // This preserves the order defined in unveil.toml.
        for slide_name in config.slides.iter() {
            let path = PathBuf::from("slides").join(slide_name);

            let mut file = File::open(&path)?;
            let mut contents = String::new();

            file.read_to_string(&mut contents)?;
            markdown_contents.push(contents);
        }

        Ok(markdown_contents)
    }

    /// Build static presentation files.
    ///
    /// Source:
    ///     ../unveil/slides/
    ///
    /// Output:
    ///     ../public/
    pub fn build(&mut self, server: &Server) -> Result<()> {
        // Read Markdown files from the current Unveil project.
        let markdowns = UnveilProject::get_markdown_from_file()?;

        let mut processor = HtmlBuilder::new(markdowns, self.livereload);
        let (user_css, html) = processor.build()?;

        // The Unveil project is located at:
        // rustcms/unveil/
        //
        // Therefore ../public points to:
        // rustcms/public/
        let public = PathBuf::from("../public");

        // Verify that this is an Unveil project.
        let config = UnveilConfig::from_disk("unveil.toml")?;

        // Create the output directory if it does not exist.
        // create_dir_all also handles missing parent directories.
        fs::create_dir_all(&public)?;

        // Preserve the original gitignore option.
        if config.gitignore {
            helper::fs::write_file(".gitignore", b"public")?;
        }

        // --------------------------------------------------
        // Main HTML and JavaScript
        // --------------------------------------------------

        fs::write(public.join("index.html"), html.as_bytes())?;
        fs::write(public.join("unveil.js"), JS)?;

        // --------------------------------------------------
        // User CSS
        // --------------------------------------------------

        if let Some(css) = user_css {
            fs::write(public.join("user_css.css"), css.as_bytes())?;
        }

        // --------------------------------------------------
        // Highlight.js and LiveReload
        // --------------------------------------------------

        fs::write(public.join("highlight.css"), HIGHLIGHT_CSS)?;

        let livereload = format!(
            r#"let socket = new WebSocket("ws://{}:{}");{}"#,
            server.hostname,
            server.ws_port,
            String::from_utf8(LIVERELOAD_JS.to_vec())?
        );

        fs::write(
            public.join("livereload.js"),
            livereload.as_bytes(),
        )?;

        fs::write(public.join("clipboard.js"), CLIPBOARD_JS)?;
        fs::write(public.join("highlight.js"), HIGHLIGHT_JS)?;

        // --------------------------------------------------
        // Font Awesome directories
        // --------------------------------------------------

        let fontawesome = public.join("fontawesome");
        let webfonts = fontawesome.join("webfonts");
        let fontawesome_css = fontawesome.join("css");

        fs::create_dir_all(&webfonts)?;
        fs::create_dir_all(&fontawesome_css)?;

        // --------------------------------------------------
        // Font Awesome Regular
        // --------------------------------------------------

        fs::write(
            webfonts.join("fa-regular-400.eot"),
            FONT_AWESOME_EOT,
        )?;

        fs::write(
            webfonts.join("fa-regular-400.svg"),
            FONT_AWESOME_SVG,
        )?;

        fs::write(
            webfonts.join("fa-regular-400.ttf"),
            FONT_AWESOME_TTF,
        )?;

        fs::write(
            webfonts.join("fa-regular-400.woff"),
            FONT_AWESOME_WOFF,
        )?;

        fs::write(
            webfonts.join("fa-regular-400.woff2"),
            FONT_AWESOME_WOFF2,
        )?;

        // --------------------------------------------------
        // Font Awesome Brands
        // --------------------------------------------------

        fs::write(
            webfonts.join("fa-brands-400.eot"),
            FONT_AWESOME_EOT_BRANDS,
        )?;

        fs::write(
            webfonts.join("fa-brands-400.svg"),
            FONT_AWESOME_SVG_BRANDS,
        )?;

        fs::write(
            webfonts.join("fa-brands-400.ttf"),
            FONT_AWESOME_TTF_BRANDS,
        )?;

        fs::write(
            webfonts.join("fa-brands-400.woff"),
            FONT_AWESOME_WOFF_BRANDS,
        )?;

        fs::write(
            webfonts.join("fa-brands-400.woff2"),
            FONT_AWESOME_WOFF2_BRANDS,
        )?;

        // --------------------------------------------------
        // Font Awesome Solid
        // --------------------------------------------------

        fs::write(
            webfonts.join("fa-solid-900.eot"),
            FONT_AWESOME_EOT_900,
        )?;

        fs::write(
            webfonts.join("fa-solid-900.svg"),
            FONT_AWESOME_SVG_900,
        )?;

        fs::write(
            webfonts.join("fa-solid-900.ttf"),
            FONT_AWESOME_TTF_900,
        )?;

        fs::write(
            webfonts.join("fa-solid-900.woff"),
            FONT_AWESOME_WOFF_900,
        )?;

        fs::write(
            webfonts.join("fa-solid-900.woff2"),
            FONT_AWESOME_WOFF2_900,
        )?;

        fs::write(
            fontawesome_css.join("fontawesome.css"),
            FONT_AWESOME,
        )?;

        // --------------------------------------------------
        // Main Unveil CSS
        // --------------------------------------------------
        //
        // Do not overwrite unveil.css if it already exists.
        // This preserves user modifications.

        let unveil_css = public.join("unveil.css");

        if !unveil_css.exists() {
            let mut css = File::create(&unveil_css)?;
            css.write_all(CSS)?;
        }

        // --------------------------------------------------
        // Theme CSS
        // --------------------------------------------------

        let theme_content = config.get_theme()?;

        fs::write(
            public.join("theme.css"),
            theme_content.as_slice(),
        )?;

        Ok(())
    }

    /// Initialize a template project.
    pub fn init(&mut self, project_name: Option<&str>) -> Result<()> {
        let project_name = if let Some(project_name) = project_name {
            project_name
        } else {
            "unveil"
        };

        // Create slides directory.
        fs::create_dir_all(
            PathBuf::from(project_name).join("slides"),
        )?;

        // Add default gitignore.
        let mut gitignore =
            File::create(PathBuf::from(project_name).join(".gitignore"))?;

        gitignore.write_all(b"public")?;

        // Add a default example slide.
        let mut landing = File::create(
            PathBuf::from(project_name)
                .join("slides")
                .join("landing.md"),
        )?;

        landing.write_all(LANDING)?;

        // Generate default configuration.
        let mut config_file = File::create(
            PathBuf::from(project_name).join("unveil.toml"),
        )?;

        let default_config = toml::to_string(&UnveilConfig::default())?;

        config_file.write_all(default_config.as_bytes())?;

        Ok(())
    }

    /// Remove files generated by Unveil.
    ///
    /// IMPORTANT:
    /// Do not remove the entire ../public directory.
    /// The parent CMS may also store static files there.
    pub fn clean() -> Result<()> {
        let public = PathBuf::from("../public");

        // Files generated by Unveil.
        let generated_files = [
            "index.html",
            "unveil.js",
            "user_css.css",
            "highlight.css",
            "livereload.js",
            "clipboard.js",
            "highlight.js",
            "unveil.css",
            "theme.css",
        ];

        for filename in generated_files {
            let path = public.join(filename);

            if path.is_file() {
                fs::remove_file(&path).map_err(|err| {
                    anyhow!(
                        "Unable to remove file {}: {}",
                        path.display(),
                        err
                    )
                })?;
            }
        }

        // Remove only the Font Awesome directory generated by Unveil.
        let fontawesome = public.join("fontawesome");

        if fontawesome.exists() {
            fs::remove_dir_all(&fontawesome).map_err(|err| {
                anyhow!(
                    "Unable to remove directory {}: {}",
                    fontawesome.display(),
                    err
                )
            })?;
        }

        // Keep ../public itself intact.
        Ok(())
    }

    /// Create a new Markdown slide.
    pub fn new_slide(&mut self, name: &str) -> Result<()> {
        let filename = if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{}.md", name)
        };

        let path = PathBuf::from("slides").join(&filename);

        let mut config = UnveilConfig::from_disk("unveil.toml")?;

        File::create(&path)?;

        config.slides.push(filename);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("unveil.toml")?;

        file.write_all(toml::to_string(&config)?.as_bytes())
            .map_err(|err| {
                anyhow!("Error writing to unveil.toml: {}", err)
            })
    }

    /// Build the presentation and start the Unveil server.
    pub async fn serve(
        &mut self,
        hostname: Option<&str>,
        http_port: Option<i32>,
        ws_port: Option<i32>,
    ) -> Result<()> {
        let server = Server::default()
            .with_hostname(hostname)
            .with_http_port(http_port)
            .with_ws_port(ws_port);

        self.build(&server)?;

        server.serve().await
    }
}
