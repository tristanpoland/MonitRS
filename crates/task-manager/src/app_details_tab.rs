use gpui::{Context, div, IntoElement, ParentElement, Render, Styled, Window, prelude::FluentBuilder};
use gpui_component::{
    h_flex, v_flex, ActiveTheme, StyledExt,
    progress::Progress,
};

use crate::system_monitor::{SystemSnapshot, format_bytes};

pub struct AppDetailsTab {
    snapshot: Option<SystemSnapshot>,
}

impl AppDetailsTab {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            snapshot: None,
        }
    }

    pub fn update_snapshot(&mut self, snapshot: SystemSnapshot, _cx: &mut Context<Self>) {
        self.snapshot = Some(snapshot);
    }

    fn render_info_card(
        &self,
        title: String,
        items: Vec<(String, String)>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .gap_3()
            .p_4()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius)
            .bg(cx.theme().background)
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .text_color(cx.theme().foreground)
                    .child(title)
            )
            .children(items.into_iter().map(|(label, value)| {
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(label)
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(cx.theme().foreground)
                            .child(value)
                    )
            }))
    }

    fn render_resource_usage(
        &self,
        label: String,
        used: u64,
        total: u64,
        color: gpui::Hsla,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let percent = if total > 0 {
            ((used as f64 / total as f64) * 100.0) as f32
        } else {
            0.0
        };

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .child(label.clone())
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{:.1}%", percent))
                    )
            )
            .child(
                Progress::new(format!("progress-{}", label))
                    .value(percent)
                    .bg(color)
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} / {}", format_bytes(used), format_bytes(total)))
            )
    }
}

impl Render for AppDetailsTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot.as_ref();

        let (cpu_info, memory_info, disk_info, network_info) = if let Some(snapshot) = snapshot {
            let cpu_count = snapshot.cpus.len();
            let avg_cpu = if cpu_count > 0 {
                snapshot.cpus.iter().map(|c| c.usage).sum::<f32>() / cpu_count as f32
            } else {
                0.0
            };

            let cpu_items = vec![
                ("Logical processors".to_string(), cpu_count.to_string()),
                ("Average usage".to_string(), format!("{:.1}%", avg_cpu)),
                ("Global usage".to_string(), format!("{:.1}%", snapshot.global_cpu_usage)),
            ];

            let memory_items = vec![
                ("Total".to_string(), format_bytes(snapshot.memory.total)),
                ("Used".to_string(), format_bytes(snapshot.memory.used)),
                ("Available".to_string(), format_bytes(snapshot.memory.available)),
            ];

            let total_disk_space: u64 = snapshot.disks.iter().map(|d| d.total).sum();
            let total_disk_available: u64 = snapshot.disks.iter().map(|d| d.available).sum();
            let disk_items = vec![
                ("Drives".to_string(), snapshot.disks.len().to_string()),
                ("Total space".to_string(), format_bytes(total_disk_space)),
                ("Available".to_string(), format_bytes(total_disk_available)),
            ];

            let total_received: u64 = snapshot.networks.iter().map(|n| n.received).sum();
            let total_transmitted: u64 = snapshot.networks.iter().map(|n| n.transmitted).sum();
            let network_items = vec![
                ("Interfaces".to_string(), snapshot.networks.len().to_string()),
                ("Total received".to_string(), format_bytes(total_received)),
                ("Total transmitted".to_string(), format_bytes(total_transmitted)),
            ];

            (cpu_items, memory_items, disk_items, network_items)
        } else {
            (vec![], vec![], vec![], vec![])
        };

        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .child("App Details")
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("System resource summary and information")
            )
            .when(snapshot.is_some(), |el| {
                let snapshot = snapshot.unwrap();
                el.child(
                    h_flex()
                        .gap_4()
                        .child(self.render_info_card("CPU".to_string(), cpu_info, cx))
                        .child(self.render_info_card("Memory".to_string(), memory_info, cx))
                )
                .child(
                    h_flex()
                        .gap_4()
                        .child(self.render_info_card("Disk".to_string(), disk_info, cx))
                        .child(self.render_info_card("Network".to_string(), network_info, cx))
                )
                .child(
                    v_flex()
                        .gap_4()
                        .p_4()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().background)
                        .child(
                            div()
                                .text_lg()
                                .font_semibold()
                                .child("Resource Usage")
                        )
                        .child(
                            self.render_resource_usage(
                                "Memory".to_string(),
                                snapshot.memory.used,
                                snapshot.memory.total,
                                cx.theme().primary,
                                cx,
                            )
                        )
                        .child({
                            let total_disk: u64 = snapshot.disks.iter().map(|d| d.total).sum();
                            let used_disk: u64 = snapshot.disks.iter().map(|d| d.total - d.available).sum();
                            self.render_resource_usage(
                                "Disk".to_string(),
                                used_disk,
                                total_disk,
                                cx.theme().warning,
                                cx,
                            )
                        })
                )
            })
    }
}
