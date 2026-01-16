use gpui::{Context, div, IntoElement, ParentElement, Render, SharedString, Styled, Window};
use gpui_component::{
    chart::{LineChart, AreaChart},
    h_flex, v_flex, ActiveTheme, StyledExt,
};
use std::collections::VecDeque;

use crate::system_monitor::{SystemSnapshot, format_bytes};

const MAX_HISTORY: usize = 60;

#[derive(Clone)]
struct DataPoint {
    time: SharedString,
    value: f64,
}

pub struct PerformanceTab {
    cpu_history: VecDeque<DataPoint>,
    memory_history: VecDeque<DataPoint>,
    disk_history: VecDeque<DataPoint>,
    network_history: VecDeque<DataPoint>,
    time_counter: u32,
    current_snapshot: Option<SystemSnapshot>,
}

impl PerformanceTab {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(MAX_HISTORY),
            memory_history: VecDeque::with_capacity(MAX_HISTORY),
            disk_history: VecDeque::with_capacity(MAX_HISTORY),
            network_history: VecDeque::with_capacity(MAX_HISTORY),
            time_counter: 0,
            current_snapshot: None,
        }
    }

    pub fn update_snapshot(&mut self, snapshot: SystemSnapshot, _cx: &mut Context<Self>) {
        self.time_counter += 1;

        let cpu_usage = snapshot.global_cpu_usage as f64;
        let memory_percent = if snapshot.memory.total > 0 {
            (snapshot.memory.used as f64 / snapshot.memory.total as f64) * 100.0
        } else {
            0.0
        };

        let total_disk: u64 = snapshot.disks.iter().map(|d| d.total - d.available).sum();
        let total_disk_capacity: u64 = snapshot.disks.iter().map(|d| d.total).sum();
        let disk_percent = if total_disk_capacity > 0 {
            (total_disk as f64 / total_disk_capacity as f64) * 100.0
        } else {
            0.0
        };

        let total_network: u64 = snapshot.networks.iter()
            .map(|n| n.received + n.transmitted)
            .sum();
        let network_mbps = (total_network as f64 / 1024.0 / 1024.0) / 1000.0;

        let time_label: SharedString = format!("{}", self.time_counter).into();

        self.cpu_history.push_back(DataPoint {
            time: time_label.clone(),
            value: cpu_usage,
        });
        self.memory_history.push_back(DataPoint {
            time: time_label.clone(),
            value: memory_percent,
        });
        self.disk_history.push_back(DataPoint {
            time: time_label.clone(),
            value: disk_percent,
        });
        self.network_history.push_back(DataPoint {
            time: time_label,
            value: network_mbps,
        });

        if self.cpu_history.len() > MAX_HISTORY {
            self.cpu_history.pop_front();
        }
        if self.memory_history.len() > MAX_HISTORY {
            self.memory_history.pop_front();
        }
        if self.disk_history.len() > MAX_HISTORY {
            self.disk_history.pop_front();
        }
        if self.network_history.len() > MAX_HISTORY {
            self.network_history.pop_front();
        }

        self.current_snapshot = Some(snapshot);
    }
}

impl Render for PerformanceTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cpu_data: Vec<DataPoint> = self.cpu_history.iter().cloned().collect();
        let memory_data: Vec<DataPoint> = self.memory_history.iter().cloned().collect();
        let disk_data: Vec<DataPoint> = self.disk_history.iter().cloned().collect();
        let network_data: Vec<DataPoint> = self.network_history.iter().cloned().collect();

        let current_cpu = cpu_data.last().map(|d| d.value).unwrap_or(0.0);
        let current_memory = memory_data.last().map(|d| d.value).unwrap_or(0.0);
        let current_disk = disk_data.last().map(|d| d.value).unwrap_or(0.0);
        let current_network = network_data.last().map(|d| d.value).unwrap_or(0.0);

        let (memory_used, memory_total) = if let Some(ref snapshot) = self.current_snapshot {
            (snapshot.memory.used, snapshot.memory.total)
        } else {
            (0, 0)
        };

        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .child("Performance")
            )
            .child(
                h_flex()
                    .flex_1()
                    .gap_4()
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child("CPU")
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(cx.theme().primary)
                                    .child(format!("{:.1}%", current_cpu))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded(cx.theme().radius)
                                    .p_2()
                                    .child(
                                        AreaChart::new(cpu_data.clone())
                                            .x(|d| d.time.clone())
                                            .y(|d| d.value)
                                            .stroke(cx.theme().primary)
                                    )
                            )
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child("Memory")
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(cx.theme().success)
                                    .child(format!("{:.1}%", current_memory))
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} / {}",
                                        format_bytes(memory_used),
                                        format_bytes(memory_total)
                                    ))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded(cx.theme().radius)
                                    .p_2()
                                    .child(
                                        AreaChart::new(memory_data.clone())
                                            .x(|d| d.time.clone())
                                            .y(|d| d.value)
                                            .stroke(cx.theme().success)
                                    )
                            )
                    )
            )
            .child(
                h_flex()
                    .flex_1()
                    .gap_4()
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child("Disk")
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(cx.theme().warning)
                                    .child(format!("{:.1}%", current_disk))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded(cx.theme().radius)
                                    .p_2()
                                    .child(
                                        AreaChart::new(disk_data.clone())
                                            .x(|d| d.time.clone())
                                            .y(|d| d.value)
                                            .stroke(cx.theme().warning)
                                    )
                            )
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child("Network")
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(cx.theme().info)
                                    .child(format!("{:.2} MB/s", current_network))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded(cx.theme().radius)
                                    .p_2()
                                    .child(
                                        LineChart::new(network_data.clone())
                                            .x(|d| d.time.clone())
                                            .y(|d| d.value)
                                            .stroke(cx.theme().info)
                                            .dot()
                                    )
                            )
                    )
            )
    }
}
