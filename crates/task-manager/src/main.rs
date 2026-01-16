mod system_monitor;
mod processes_tab;
mod performance_tab;
mod app_details_tab;

use gpui::{
    actions, Application, App, AppContext, Bounds, Context, div, Entity, IntoElement, KeyBinding,
    ParentElement, Render, Styled, Task, Window, WindowBounds, WindowOptions, px, size,
    prelude::FluentBuilder, InteractiveElement,
};
use gpui_component::{
    v_flex, tab::{Tab, TabBar}, ActiveTheme, Root, StyledExt,
};

use system_monitor::SystemMonitor;
use processes_tab::ProcessesTab;
use performance_tab::PerformanceTab;
use app_details_tab::AppDetailsTab;

actions!(task_manager, [Quit]);

const CONTEXT: &str = "TaskManager";

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Processes,
    Performance,
    AppDetails,
}

struct TaskManagerApp {
    active_tab: ActiveTab,
    monitor: SystemMonitor,
    processes_tab: Entity<ProcessesTab>,
    performance_tab: Entity<PerformanceTab>,
    app_details_tab: Entity<AppDetailsTab>,
    update_task: Option<Task<()>>,
}

impl TaskManagerApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let monitor = SystemMonitor::new();
        let snapshot = monitor.snapshot();

        let processes_tab = cx.new(|cx| {
            ProcessesTab::new(snapshot.processes.clone(), window, cx)
        });

        let performance_tab = cx.new(|cx| {
            let mut tab = PerformanceTab::new(cx);
            tab.update_snapshot(snapshot.clone(), cx);
            tab
        });

        let app_details_tab = cx.new(|cx| {
            let mut tab = AppDetailsTab::new(cx);
            tab.update_snapshot(snapshot.clone(), cx);
            tab
        });

        let mut app = Self {
            active_tab: ActiveTab::Processes,
            monitor,
            processes_tab,
            performance_tab,
            app_details_tab,
            update_task: None,
        };

        app.start_monitoring(cx);
        app
    }

    fn start_monitoring(&mut self, cx: &mut Context<Self>) {
        let processes_tab = self.processes_tab.clone();
        let performance_tab = self.performance_tab.clone();
        let app_details_tab = self.app_details_tab.clone();

        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_secs(1)).await;

                let _ = this.update(cx, |this, cx| {
                    this.monitor.update();
                    let snapshot = this.monitor.snapshot();

                    processes_tab.update(cx, |tab, cx| {
                        tab.update_processes(snapshot.processes.clone(), cx);
                    });

                    performance_tab.update(cx, |tab, cx| {
                        tab.update_snapshot(snapshot.clone(), cx);
                    });

                    app_details_tab.update(cx, |tab, cx| {
                        tab.update_snapshot(snapshot.clone(), cx);
                    });

                    cx.notify();
                });
            }
        });

        self.update_task = Some(task);
    }

    fn set_active_tab(&mut self, tab: ActiveTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    fn quit(&mut self, _action: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for TaskManagerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_index = match self.active_tab {
            ActiveTab::Processes => 0,
            ActiveTab::Performance => 1,
            ActiveTab::AppDetails => 2,
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::quit))
            .child(
                div()
                    .p_4()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_2xl()
                            .font_bold()
                            .child("Task Manager")
                    )
            )
            .child(
                TabBar::new("main-tabs")
                    .selected_index(active_index)
                    .on_click(cx.listener(move |this: &mut Self, ix: &usize, _window, cx| {
                        let tab = match ix {
                            0 => ActiveTab::Processes,
                            1 => ActiveTab::Performance,
                            2 => ActiveTab::AppDetails,
                            _ => return,
                        };
                        this.set_active_tab(tab, cx);
                    }))
                    .child(Tab::new().child("Processes"))
                    .child(Tab::new().child("Performance"))
                    .child(Tab::new().child("App Details"))
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .when(self.active_tab == ActiveTab::Processes, |el| {
                        el.child(self.processes_tab.clone())
                    })
                    .when(self.active_tab == ActiveTab::Performance, |el| {
                        el.child(self.performance_tab.clone())
                    })
                    .when(self.active_tab == ActiveTab::AppDetails, |el| {
                        el.child(self.app_details_tab.clone())
                    })
            )
    }
}

fn main() {
    env_logger::init();

    let app = Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, Some(CONTEXT)),
            KeyBinding::new("ctrl-q", Quit, Some(CONTEXT)),
        ]);

        let window_size = size(px(1200.0), px(800.0));
        let window_bounds = Bounds::centered(None, window_size, cx);

        let _window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Task Manager".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_background: gpui::WindowBackgroundAppearance::Opaque,
                focus: true,
                show: true,
                kind: gpui::WindowKind::Normal,
                is_movable: true,
                is_minimizable: true,
                is_resizable: true,
                display_id: None,
                window_min_size: Some(size(px(800.0), px(600.0))),
                app_id: Some("com.taskmanager.app".to_string()),
                tabbing_identifier: None,
                window_decorations: Some(gpui::WindowDecorations::Client),
            },
            |window, cx| {
                let view = TaskManagerApp::view(window, cx);
                cx.new(|cx| Root::new(view, window, cx))
            },
        ).ok();

        cx.activate(true);
    });
}
