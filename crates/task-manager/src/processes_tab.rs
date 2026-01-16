use gpui::{App, AppContext, Context, div, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{
    table::{TableDelegate, TableState, Table, Column, ColumnSort},
    v_flex, StyledExt,
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
    sort_column: ProcessColumn,
    sort_ascending: bool,
}

impl ProcessesTableDelegate {
    pub fn new(processes: Vec<ProcessInfo>) -> Self {
        let mut delegate = Self {
            processes,
            sort_column: ProcessColumn::Cpu,
            sort_ascending: false,
        };
        delegate.sort();
        delegate
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
        self.sort();
    }

    fn sort(&mut self) {
        match self.sort_column {
            ProcessColumn::Name => {
                self.processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            ProcessColumn::Pid => {
                self.processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.pid.cmp(&b.pid)
                    } else {
                        b.pid.cmp(&a.pid)
                    }
                });
            }
            ProcessColumn::Cpu => {
                self.processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                    }
                });
            }
            ProcessColumn::Memory => {
                self.processes.sort_by(|a, b| {
                    if self.sort_ascending {
                        a.memory.cmp(&b.memory)
                    } else {
                        b.memory.cmp(&a.memory)
                    }
                });
            }
            ProcessColumn::Disk => {
                self.processes.sort_by(|a, b| {
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
        self.processes.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        let all_columns = ProcessColumn::all();
        let col = all_columns.get(col_ix).unwrap();
        let width = match col {
            ProcessColumn::Name => 250.0,
            ProcessColumn::Pid => 100.0,
            ProcessColumn::Cpu => 120.0,
            ProcessColumn::Memory => 150.0,
            ProcessColumn::Disk => 150.0,
        };

        Column::new(col.key(), col.label())
            .width(width)
            .sortable()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let process = &self.processes[row_ix];
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
        _cx: &mut Context<TableState<Self>>,
    ) {
        if let Some(column) = ProcessColumn::all().get(col_ix) {
            self.sort_column = *column;
            self.sort_ascending = match sort {
                ColumnSort::Ascending => true,
                ColumnSort::Descending => false,
                ColumnSort::Default => false,
            };
            self.sort();
        }
    }
}

pub struct ProcessesTab {
    table_state: Entity<TableState<ProcessesTableDelegate>>,
}

impl ProcessesTab {
    pub fn new(processes: Vec<ProcessInfo>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ProcessesTableDelegate::new(processes);
        let table_state = cx.new(|cx| TableState::new(delegate, window, cx));

        Self { table_state }
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
                div()
                    .text_xl()
                    .font_semibold()
                    .child("Processes")
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
