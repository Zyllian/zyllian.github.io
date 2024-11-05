mod builder;
mod extras;
mod frontmatter;
mod link_list;
mod resource;
#[cfg(feature = "serve")]
pub mod serving;
mod util;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use extras::ExtraData;
use eyre::Context;
use rayon::prelude::*;
use resource::{EmbedMetadata, ResourceBuilderConfig};
use serde::Deserialize;
use url::Url;
use util::get_name;
use walkdir::WalkDir;

use builder::SiteBuilder;

const PAGES_PATH: &str = "pages";
const TEMPLATES_PATH: &str = "templates";
const SASS_PATH: &str = "sass";
const ROOT_PATH: &str = "root";

/// Struct for the site's configuration.
#[derive(Debug, Deserialize)]
pub struct SiteConfig {
	/// The location the site is at.
	pub base_url: Url,
	/// The site's title.
	pub title: String,
	/// The site's description? Not sure if this will actually be used or not
	pub description: String,
	/// The site's build directory. Defaults to <site>/build if not specified.
	pub build: Option<String>,
	/// A list of Sass stylesheets that will be built.
	pub sass_styles: Vec<PathBuf>,
	/// URL to the CDN used for the site's images.
	pub cdn_url: Url,
	/// List of resources the site should build.
	pub resources: HashMap<String, ResourceBuilderConfig>,
}

impl SiteConfig {
	/// Gets a CDN url from the given file name.
	pub fn cdn_url(&self, file: &str) -> eyre::Result<Url> {
		Ok(self.cdn_url.join(file)?)
	}
}

/// Struct for the front matter in templates. (nothing here yet)
#[derive(Debug, Default, Deserialize)]
pub struct TemplateMetadata {}

/// Struct for the front matter in pages.
#[derive(Debug, Default, Deserialize)]
pub struct PageMetadata {
	/// The page's title.
	pub title: Option<String>,
	/// The template to use for the page. If not specified, it defaults to "base".
	pub template: Option<String>,
	/// custom embed info for a template
	#[serde(default)]
	pub embed: Option<EmbedMetadata>,
	/// The page's custom scripts, if any.
	#[serde(default)]
	pub scripts: Vec<String>,
	/// the page's custom styles, if any.
	#[serde(default)]
	pub styles: Vec<String>,
	/// The extra stuff to run for the page, if any.
	#[serde(default)]
	pub extra: Option<ExtraData>,
}

/// Struct containing information about the site.
#[derive(Debug)]
pub struct Site {
	/// The path to the site.
	pub site_path: PathBuf,
	/// The site's configuration.
	pub config: SiteConfig,
	/// An index of available templates.
	pub template_index: HashMap<String, PathBuf>,
	/// An index of available pages.
	pub page_index: HashMap<String, PathBuf>,
}

impl Site {
	/// Creates a new site from the given path.
	pub fn new(site_path: &Path) -> eyre::Result<Self> {
		let config: SiteConfig = serde_yml::from_str(
			&std::fs::read_to_string(site_path.join("config.yaml"))
				.wrap_err("Failed to read site config")?,
		)
		.wrap_err("Failed to parse site config")?;

		let mut template_index = HashMap::new();
		let templates_path = site_path.join(TEMPLATES_PATH);
		for entry in WalkDir::new(&templates_path).into_iter() {
			let entry = entry.wrap_err("Failed to read template entry")?;
			let path = entry.path();

			if let Some(ext) = path.extension() {
				if ext == "hbs" && entry.file_type().is_file() {
					let (_, name) = get_name(
						path.strip_prefix(&templates_path)
							.wrap_err("This really shouldn't have happened")?,
					);
					template_index.insert(name, path.to_owned());
				}
			}
		}

		let mut page_index = HashMap::new();
		let pages_path = site_path.join(PAGES_PATH);
		for entry in WalkDir::new(&pages_path).into_iter() {
			let entry = entry.wrap_err("Failed to read page entry")?;
			let path = entry.path();

			if let Some(ext) = path.extension() {
				if ext == "md" && entry.file_type().is_file() {
					page_index.insert(
						path.strip_prefix(&pages_path)
							.wrap_err("This really shouldn't have happened")?
							.with_extension("")
							.to_string_lossy()
							.to_string(),
						path.to_owned(),
					);
				}
			}
		}

		Ok(Self {
			site_path: site_path.to_owned(),
			config,
			template_index,
			page_index,
		})
	}

	/// Builds the site once.
	pub fn build_once(self) -> eyre::Result<()> {
		let builder = SiteBuilder::new(self, false).prepare()?;

		builder.site.build_all_pages(&builder)?;
		builder.build_sass()?;

		for (_source_path, config) in builder.site.config.resources.iter() {
			let mut res_builder = resource::ResourceBuilder::new(config.clone());
			res_builder.load_all(&builder)?;
			res_builder.build_all(&builder)?;
		}

		Ok(())
	}

	/// Helper method to build all available pages.
	fn build_all_pages(&self, builder: &SiteBuilder) -> eyre::Result<()> {
		self.page_index
			.keys()
			.par_bridge()
			.try_for_each(|page_name| builder.build_page(page_name))?;

		Ok(())
	}
}
