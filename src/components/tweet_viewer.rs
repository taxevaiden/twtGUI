use iced::{
    Background, Border, Color, ContentFit, Element, Font, Length, Theme,
    border::Radius,
    widget::{Image, column, container, image::Handle, markdown, rich_text, space},
};

pub struct TweetViewer<'b> {
    image_urls: &'b [String],
    image_handles: &'b [Option<(Handle, u32, u32)>],
}

impl<'b> TweetViewer<'b> {
    pub fn new(image_urls: &'b [String], image_handles: &'b [Option<(Handle, u32, u32)>]) -> Self {
        Self {
            image_urls,
            image_handles,
        }
    }
}

impl<'a, 'b, Renderer> markdown::Viewer<'a, String, iced::Theme, Renderer> for TweetViewer<'b>
where
    'b: 'a,
    Renderer: iced_core::text::Renderer<Font = Font> + 'a,
    Renderer: iced_core::image::Renderer<Handle = Handle>,
{
    fn on_link_click(url: String) -> String {
        url
    }

    fn image(
        &self,
        settings: markdown::Settings,
        url: &'a String,
        _title: &'a str,
        alt: &markdown::Text,
    ) -> Element<'a, String, Theme, Renderer> {
        const MAX_WIDTH: f32 = 500.0;
        const MAX_HEIGHT: f32 = 500.0;

        let maybe_handle = self
            .image_urls
            .iter()
            .position(|u| u == url)
            .and_then(|i| self.image_handles.get(i))
            .and_then(|h| h.as_ref());

        let image: Element<'a, String, iced::Theme, Renderer> = match maybe_handle {
            Some((handle, img_w, img_h)) if *img_w > 0 && *img_h > 0 => {
                let aspect = *img_h as f32 / *img_w as f32;
                let render_h = (MAX_WIDTH * aspect).min(MAX_HEIGHT);
                Image::<Handle>::new(handle.clone())
                    .width(Length::Fixed(MAX_WIDTH))
                    .height(Length::Fixed(render_h))
                    .content_fit(ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
                    .into()
            }
            Some((handle, _, _)) => Image::<Handle>::new(handle.clone())
                .width(Length::Fill)
                .height(Length::Shrink)
                .content_fit(ContentFit::Contain)
                .filter_method(iced::widget::image::FilterMethod::Linear)
                .into(),
            None => space::horizontal().width(0).into(),
        };

        container(column![rich_text(alt.spans(settings.style)), image].spacing(8))
            .padding(8)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(Background::from(theme.palette().background)),
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: Radius::new(8),
                },
                ..Default::default()
            })
            .into()
    }
}
