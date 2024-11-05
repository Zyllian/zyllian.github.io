use std::borrow::Cow;

use serde::Serialize;

use crate::{builder::SiteBuilder, PageMetadata};

/// Helper for links.
#[derive(Debug, Serialize)]
pub struct Link<'l> {
	/// The link's actual link.
	pub link: Cow<'l, str>,
	/// The link's title.
	pub title: Cow<'l, str>,
}

impl<'l> Link<'l> {
	/// Creates a new link.
	pub fn new(link: impl Into<Cow<'l, str>>, title: impl Into<Cow<'l, str>>) -> Self {
		let link = link.into();
		let title = title.into();
		Self { link, title }
	}
}

/// Renders a basic list of links.
pub fn render_basic_link_list(
	builder: &SiteBuilder,
	links: Vec<Link>,
	title: &str,
) -> eyre::Result<String> {
	let data = LinkTemplateData { links, title };
	let out = builder.tera.render(
		"basic-link-list.tera",
		&tera::Context::from_serialize(data)?,
	)?;
	let out = builder.build_page_raw(
		PageMetadata {
			title: Some(title.to_owned()),
			..Default::default()
		},
		&out,
	)?;
	Ok(out)
}

/// Template data for a list of links.
#[derive(Debug, Serialize)]
struct LinkTemplateData<'l> {
	/// The actual links.
	links: Vec<Link<'l>>,
	/// The title for the page.
	title: &'l str,
}
