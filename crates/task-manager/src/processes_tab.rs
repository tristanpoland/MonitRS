use gpui::{App, AppContext, Context, div, Entity, IntoElement, ParentElement, Render, Styled, Window, Subscription};
use gpui_component::{
    table::{TableDelegate, TableState, Table, Column, ColumnSort},
    input::{InputState, Input, InputEvent},
    v_flex, h_flex, StyledExt,
};

use crate::system_monitor::{ProcessInfo, format_bytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessColumn {
    Name,
    Pid,
    Cpu,
    Memory,
    Disk,
}

impl ProcessColumn {
    fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Pid => "PID",
            Self::Cpu => "CPU %",
            Self::Memory => "Memory",
            Self::Disk => "Disk",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Pid => "pid",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Disk => "disk",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Name,
            Self::Pid,
            Self::Cpu,
            Self::Memory,
            Self::Disk,
        ]
    }
}

pub struct ProcessesTableDelegate {
    processes: Vec<ProcessInfo>,
    filtered_processes: Vec<ProcessInfo>,
    filter_query: String,
    sort_column: ProcessColumn,
    sort_ascending: bool,
    columns: Vec<Column>,
}

impl ProcessesTableDelegate {
    pub fn new(processes: Vec<ProcessInfo>) -> Self {
        let columns = vec![
            Column::new("name", "Name").width(250.0).sortable(),
            Column::new("pid", "PID").width(100.0).sortable(),
            Column::new("cpu", "CPU %").width(120.0).sortable(),
            Column::new("memory", "Memory").width(150.0).sortable(),
            Column::new("disk", "Disk").width(150.0).sortable(),
        ];

        let mut delegate = Self {
            processes,
            filtered_processes: Vec::new(),
            filter_query: String::new(),
            sort_column: ProcessColumn::Cpu,
            sort_ascending: false,
            columns,
        };
        delegate.apply_filter();
        delegate.sort();
        delegate
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
        self.apply_filter();
        self.sort();
    }

    pub fn set_filter(&mut self, query: String) {
        self.filter_query = query.to_lowercase();
        self.apply_filter();
        self.sort();
    }

    fn apply_filter(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered_processes = self.processes.clone();
        } else {
            self.filtered_processes = self.processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&self.filter_query) ||
                    p.pid.to_string().contains(&self.filter_query)
                })
                .cloned()
                .collect();
        }
    }

    fn sort(&mut self) {
        match self.sort_column {
            ProcessColumn::Name => {
                self.filtered_processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            ProcessColumn::Pid => {
                self.filtered_processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.pid.cmp(&b.pid)
                    } else {
                        b.pid.cmp(&a.pid)
                    }
                });
            }
            ProcessColumn::Cpu => {
                self.filtered_processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                    }
                });
            }
            ProcessColumn::Memory => {
                self.filtered_processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.memory.cmp(&b.memory)
                    } else {
                        b.memory.cmp(&a.memory)
                    }
                });
            }
            ProcessColumn::Disk => {
                self.filtered_processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.disk_usage.cmp(&b.disk_usage)
                    } else {
                        b.disk_usage.cmp(&a.disk_usage)
                    }
                });
            }
        }
    }
}

impl TableDelegate for ProcessesTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        ProcessColumn::all().len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.filtered_processes.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let process = &self.filtered_processes[row_ix];
        let all_columns = ProcessColumn::all();
        let column = all_columns.get(col_ix).unwrap();

        let text = match column {
            ProcessColumn::Name => process.name.clone(),
            ProcessColumn::Pid => process.pid.to_string(),
            ProcessColumn::Cpu => format!("{:.1}%", process.cpu_usage),
            ProcessColumn::Memory => format_bytes(process.memory),
            ProcessColumn::Disk => format_bytes(process.disk_usage),
        };

        div().child(text)
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if let Some(column) = ProcessColumn::all().get(col_ix) {
            self.sort_column = *column;
            self.sort_ascending = match sort {
                ColumnSort::Ascending => true,
                ColumnSort::Descending => false,
                ColumnSort::Default => false,
            };
            self.sort();
            cx.notify();
        }
    }
}

pub struct ProcessesTab {
    table_state: Entity<TableState<ProcessesTableDelegate>>,
    search_input: Entity<InputState>,
    _subscription: Subscription,
}

impl ProcessesTab {
    pub fn new(processes: Vec<ProcessInfo>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ProcessesTableDelegate::new(processes);
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .sortable(true)
        });

        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search processes by name or PID...")
        });

        let _subscription = cx.subscribe_in(&search_input, window, Self::on_search_input);

        Self {
            table_state,
            search_input,
            _subscription,
        }
    }

    fn on_search_input(&mut self, _: &Entity<InputState>, _event: &InputEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let query = self.search_input.read(cx).value();
        self.table_state.update(cx, |state, _cx| {
            state.delegate_mut().set_filter(query.to_string());
        });
        cx.notify();
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>, cx: &mut App) {
        self.table_state.update(cx, |state, _cx| {
            state.delegate_mut().update_processes(processes);
        });
    }
}

impl Render for ProcessesTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xl()
                            .font_semibold()
                            .child("Processes")
                    )
                    .child(
                        div()
                            .w_64()
                            .child(Input::new(&self.search_input))
                    )
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        Table::new(&self.table_state)
                            .stripe(true)
                            .bordered(true)
                    )
            )
    }
}
