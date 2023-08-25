use iced::Color;
use iced::Element;
use iced::Length;

use iced::theme;
use iced::widget::scrollable::Id;
use iced::widget::{
    button, column, container, image, pick_list, row, scrollable, text, text_input, Button,
    Checkbox, Rule, Slider, Space,
};

use super::Krustmote;
use super::BLANK_IMAGE;
use super::ITEM_HEIGHT;
use super::{ListData, Message, Modals, State};

use crate::icons;
use crate::koditypes::*;
use crate::themes;
// use crate::themes;

use chrono;

pub(crate) fn make_subtitle_modal<'a>(
    krustmote: &'a Krustmote,
) -> iced::widget::Container<'a, Message> {
    container(column![
        row![
            text("Subtitles").height(40),
            Space::new(Length::Fill, 10),
            // This is likely the only place this Message is used
            // however it's the only way I can think to do this
            button("Download").on_press(Message::HideModalAndKodiReq(
                KodiCommand::GUIActivateWindow("subtitlesearch")
            )),
        ],
        Rule::horizontal(5),
        row![
            pick_list(
                &krustmote.kodi_status.subtitles,
                krustmote.kodi_status.current_subtitle.clone(),
                Message::SubtitlePicked
            )
            .placeholder("No Subtitles")
            .width(Length::Fill),
            Checkbox::new(
                "",
                krustmote.kodi_status.subtitles_enabled,
                Message::SubtitleEnable
            ),
        ],
        row![
            button("-").on_press(Message::KodiReq(KodiCommand::InputExecuteAction(
                "subtitledelayminus"
            ))),
            text(" Delay "),
            button("+").on_press(Message::KodiReq(KodiCommand::InputExecuteAction(
                "subtitledelayplus"
            )))
        ]
        .align_items(iced::Alignment::Center),
        // Subtitle adjust buttons.
    ])
    .width(300)
    .padding(10)
    .style(theme::Container::Box)
}

pub(crate) fn request_text_modal<'a>(
    krustmote: &'a Krustmote,
) -> iced::widget::Container<'a, Message> {
    container(column![
        text_input("Text to send...", &krustmote.send_text)
            .on_input(Message::SendTextInput)
            .on_submit(Message::HideModalAndKodiReq(KodiCommand::InputSendText(
                krustmote.send_text.clone()
            ))),
        row![
            Space::new(Length::Fill, 5),
            button("Send").on_press(Message::HideModalAndKodiReq(KodiCommand::InputSendText(
                krustmote.send_text.clone()
            ))),
        ]
    ])
    .width(300)
    .padding(10)
    .style(theme::Container::Box)
}

pub(crate) fn playing_bar<'a>(krustmote: &Krustmote) -> Element<'a, Message> {
    let bare = themes::ColoredButton::Bare;

    let duration = krustmote.kodi_status.duration.total_seconds();
    let play_time = krustmote.kodi_status.play_time.total_seconds();
    let timeleft = duration.saturating_sub(play_time);
    let now = chrono::offset::Local::now();
    let end = now + chrono::Duration::seconds(timeleft as i64);
    let end = end.format("%I:%M %p");
    if krustmote.kodi_status.now_playing {
        container(
            row![
                Space::new(5, 5),
                column![
                    Slider::new(0..=duration, play_time, Message::SliderChanged)
                        .on_release(Message::SliderReleased),
                    row![
                        text(format!("{}", krustmote.kodi_status.play_time,)).size(14),
                        Space::new(Length::Fill, 5),
                        text(format!("{} ({end})", krustmote.kodi_status.duration)).size(14),
                    ],
                    text(krustmote.kodi_status.playing_title.clone()),
                ]
                .width(Length::FillPortion(60)),
                row![
                    Space::new(Length::Fill, 5),
                    button(icons::skip_previous().size(32).height(48))
                        .style(theme::Button::custom(bare)),
                    button(icons::fast_rewind().size(32).height(48))
                        .style(theme::Button::custom(bare)),
                    button(if !krustmote.kodi_status.paused {
                        icons::pause_clircle_filled().size(48)
                    } else {
                        icons::play_circle_filled().size(48)
                    })
                    .on_press(Message::KodiReq(KodiCommand::InputExecuteAction(
                        "playpause"
                    )))
                    .style(theme::Button::custom(bare)),
                    button(icons::fast_forward().size(32).height(48))
                        .style(theme::Button::custom(bare)),
                    button(icons::skip_next().size(32).height(48))
                        .style(theme::Button::custom(bare)),
                    button(icons::stop().size(32).height(48))
                        .on_press(Message::KodiReq(KodiCommand::InputExecuteAction("stop")))
                        .style(theme::Button::custom(bare)),
                    Space::new(20, 5),
                    column![
                        button(icons::subtitles())
                            .on_press(Message::ShowModal(Modals::Subtitles))
                            .style(theme::Button::custom(bare)),
                        button(icons::smart_display()).style(theme::Button::custom(bare)),
                        button(icons::hearing()).style(theme::Button::custom(bare)),
                    ],
                    Space::new(10, 5),
                ]
                .width(Length::FillPortion(40))
                .align_items(iced::Alignment::Center)
            ]
            .spacing(20),
        )
        .height(80)
        .into()
    } else {
        container(Space::new(0, 0)).into()
    }
}

pub(crate) fn top_bar<'a>(krustmote: &Krustmote) -> Element<'a, Message> {
    container(row![
        button("=")
            .on_press(Message::ToggleLeftMenu)
            .style(theme::Button::custom(themes::ColoredButton::Bare)),
        Space::new(Length::Fill, Length::Shrink),
        text_input("Filter..", &krustmote.item_list.filter).on_input(Message::FilterFileList),
        match krustmote.state {
            State::Disconnected => icons::sync_disabled(),
            _ => icons::sync(),
        },
    ])
    .into()
}

pub(crate) fn center_area<'a>(krustmote: &'a Krustmote) -> Element<'a, Message> {
    let offset = krustmote.item_list.start_offset;

    let count =
        (offset + krustmote.item_list.visible_count).min(krustmote.item_list.data.len() as u32);

    let mut virtual_list: Vec<Element<'a, Message>> = Vec::new();

    let top_space = offset * ITEM_HEIGHT;
    virtual_list.push(Space::new(10, top_space as f32).into());

    let mut precount = 0;
    let files = krustmote
        .item_list
        .data
        .iter()
        .filter(|&x| {
            x.label
                .to_lowercase()
                .contains(&krustmote.item_list.filter.to_lowercase())
        })
        .enumerate()
        .filter(|&(i, _)| {
            precount = i;
            i as u32 >= offset && i as u32 <= count
        })
        .map(|(_, data)| make_listitem(data))
        .map(Element::from);

    virtual_list.extend(files);

    let bottom_space = if !krustmote.item_list.filter.is_empty() {
        precount as u32 * ITEM_HEIGHT
    } else {
        krustmote.item_list.data.len() as u32 * ITEM_HEIGHT
    }
    .saturating_sub(offset * ITEM_HEIGHT)
    .saturating_sub(krustmote.item_list.visible_count * ITEM_HEIGHT);

    virtual_list.push(Space::new(10, bottom_space as f32).into());

    // dbg!(virtual_list.len());

    let virtual_list = column(virtual_list);

    column![
        row![if krustmote.item_list.breadcrumb.len() > 1 {
            button("..")
                .on_press(Message::UpBreadCrumb)
                .width(Length::Fill)
                .height(50)
                .style(iced::theme::Button::custom(themes::ColoredButton::ListItem))
        } else {
            button("")
                .width(Length::Fill)
                .height(50)
                .style(iced::theme::Button::custom(themes::ColoredButton::ListItem))
        },]
        .spacing(1)
        .padding(iced::Padding {
            left: 5.0,
            top: 5.0,
            right: 0.0,
            bottom: 5.0
        }),
        scrollable(virtual_list.spacing(1).padding(iced::Padding {
            left: 5.0,
            top: 5.0,
            right: 5.0,
            bottom: 5.0
        }),)
        .on_scroll(Message::Scrolled)
        .id(Id::new("files"))
    ]
    .width(Length::Fill)
    .into()
}

pub(crate) fn make_listitem(data: &ListData) -> Button<Message> {
    // Let's stretch the definition of a 'button'
    // ___________________________________________________________
    // | picture |  Main Label Information                       |
    // | picture |  (smaller text) content (genre, runtime, etc) |
    // | picture |  bottom left                     bottom right |
    // -----------------------------------------------------------
    //
    // row![ picture, column! [ label,
    //                          content,
    //                          row! [bottom_left, space, bottom_right],
    //                         ]
    //     ]
    // It seems pretty clear I'll have to make some kind of custom
    //    RecyclerView type thing.
    //    The button captures any attempt to touch-scroll.
    //    and there's no 'fling' anyway
    //
    // TODO: I should specify label heights here to ensure no line wrapping/etc
    let image_data = data.image.get();
    Button::new(row![
        if let Some(img) = image_data {
            container(image(img.clone()).height(45))
        } else {
            // Could use a Space here instead
            //   but BLANK_IMAGE will eventually be PLACEHOLDER_IMAGE
            container(image(BLANK_IMAGE.get().unwrap().clone()).height(45))
        },
        // Watched will proabbly go in picture area - for now just this icon or not
        if data.play_count.unwrap_or(0) > 0 {
            icons::done()
        } else {
            text(" ")
        },
        column![
            text(data.label.as_str()).size(14).height(18),
            text("").size(10),
            row![
                match &data.bottom_left {
                    Some(d) => text(d.as_str()).size(10),
                    None => text(""),
                },
                Space::new(Length::Fill, Length::Shrink),
                match &data.bottom_right {
                    Some(d) => text(d.as_str()).size(10),
                    None => text(""),
                },
            ]
        ]
    ])
    .on_press(data.on_click.clone())
    .width(Length::Fill)
    .height(ITEM_HEIGHT as f32)
    .style(theme::Button::custom(themes::ColoredButton::ListItem))
}

pub(crate) fn left_menu<'a>(krustmote: &Krustmote) -> Element<'a, Message> {
    let bare = themes::ColoredButton::Bare;
    container(
        column![
            button(row![icons::folder(), "Files"])
                .on_press(Message::KodiReq(KodiCommand::GetSources(MediaType::Video)))
                .width(Length::Fill)
                .style(theme::Button::custom(bare)),
            button("Settings")
                .width(Length::Fill)
                .style(theme::Button::custom(bare)),
        ]
        .spacing(1)
        .padding(5)
        .width(100),
    )
    .max_width(krustmote.menu_width)
    .into()
}

pub(crate) fn remote<'a>(krustmote: &Krustmote) -> Element<'a, Message> {
    let red = Color::from_rgb8(255, 0, 0);
    container(
        column![
            // seems like I could template these buttons in some way
            button(icons::bug_report()).on_press(Message::KodiReq(KodiCommand::Test)),
            button("playerid-test").on_press(Message::KodiReq(KodiCommand::PlayerGetActivePlayers)),
            button("props-test").on_press(Message::KodiReq(KodiCommand::PlayerGetProperties)),
            button("item-test").on_press(Message::KodiReq(KodiCommand::PlayerGetPlayingItem(
                krustmote.kodi_status.active_player_id.unwrap_or(0)
            ))),
            row![
                button(icons::volume_down().size(32)).on_press(Message::KodiReq(
                    KodiCommand::InputExecuteAction("volumedown")
                )),
                if krustmote.kodi_status.muted {
                    button(icons::volume_off().style(red).size(32))
                        .on_press(Message::KodiReq(KodiCommand::ToggleMute))
                } else {
                    button(icons::volume_off().size(32))
                        .on_press(Message::KodiReq(KodiCommand::ToggleMute))
                },
                button(icons::volume_up().size(32)).on_press(Message::KodiReq(
                    KodiCommand::InputExecuteAction("volumeup")
                )),
            ]
            .spacing(10),
            row![
                button(icons::fullscreen().size(30)).on_press(Message::KodiReq(
                    KodiCommand::InputButtonEvent {
                        button: "display",
                        keymap: "R1"
                    }
                )),
                button(icons::info().size(30)).on_press(Message::KodiReq(
                    KodiCommand::InputButtonEvent {
                        button: "info",
                        keymap: "R1"
                    }
                )),
                button(icons::keyboard().size(30))
                    .on_press(Message::ShowModal(Modals::RequestText)),
            ]
            .spacing(10),
            row![
                button(icons::call_to_action().size(30)).on_press(Message::KodiReq(
                    KodiCommand::InputButtonEvent {
                        button: "menu",
                        keymap: "R1"
                    }
                )),
                Space::new(40, 40), // Not sure what to put here.
                button(icons::format_list_bulleted().size(30)).on_press(Message::KodiReq(
                    KodiCommand::InputButtonEvent {
                        button: "title",
                        keymap: "R1"
                    }
                )),
            ]
            .spacing(10),
            Space::new(Length::Shrink, Length::Fill),
            row![
                // Might add pgup/pgdn buttons on either side here.
                Space::new(65, 65),
                button(icons::expand_less().size(48))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "up",
                        keymap: "R1",
                    })),
                Space::new(65, 65),
            ]
            .spacing(5),
            row![
                button(icons::chevron_left().size(48))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "left",
                        keymap: "R1",
                    })),
                button(icons::circle().size(48))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "select",
                        keymap: "R1",
                    })),
                button(icons::chevron_right().size(48))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "right",
                        keymap: "R1",
                    })),
            ]
            .spacing(5),
            row![
                button(icons::arrow_back().size(32))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "back",
                        keymap: "R1",
                    })),
                button(icons::expand_more().size(48))
                    .width(65)
                    .height(65)
                    .on_press(Message::KodiReq(KodiCommand::InputButtonEvent {
                        button: "down",
                        keymap: "R1",
                    })),
                Space::new(65, 65),
            ]
            .spacing(5),
        ]
        .padding(10)
        .spacing(5)
        .align_items(iced::Alignment::Center),
    )
    .width(230)
    .into()
}
