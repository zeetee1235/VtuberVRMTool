use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BoneInfo {
    name: String,
    parent_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmrInfo {
    name: String,
    root_bone: Option<String>,
    bones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VrmInput {
    avatar_bones: Vec<BoneInfo>,
    clothing_bones: Vec<BoneInfo>,
    clothing_smrs: Vec<SmrInfo>,
    suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VrmReport {
    duplicate_avatar_bone_names: Vec<String>,
    duplicate_clothing_bone_names: Vec<String>,
    referenced_clothing_bones: usize,
    estimated_moved_bones: usize,
    estimated_moved_smrs: usize,
    estimated_renamed_bones: usize,
    estimated_renamed_smrs: usize,
    warnings: Vec<String>,
}

#[derive(Clone, Copy)]
enum Action {
    CheckDuplicateBoneNames,
    BuildDryRunPlan,
}

impl Action {
    fn title(self) -> &'static str {
        match self {
            Action::CheckDuplicateBoneNames => "본 이름 중복 검사",
            Action::BuildDryRunPlan => "병합 드라이런 계획 생성",
        }
    }
}

struct App {
    actions: Vec<Action>,
    selected: usize,
    logs: Vec<String>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    input_data: Option<VrmInput>,
    latest_report: Option<VrmReport>,
}

impl App {
    fn new(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Self {
        let mut logs = vec!["vrm-tui 시작. 방향키/Enter로 실행, s로 저장, q로 종료".to_string()];

        if let Some(path) = &input_path {
            logs.push(format!("입력 파일: {}", path.display()));
        } else {
            logs.push("입력 파일이 없어 예시 데이터 모드로 동작합니다.".to_string());
        }

        if let Some(path) = &output_path {
            logs.push(format!("출력 파일: {}", path.display()));
        }

        Self {
            actions: vec![Action::CheckDuplicateBoneNames, Action::BuildDryRunPlan],
            selected: 0,
            logs,
            input_path,
            output_path,
            input_data: None,
            latest_report: None,
        }
    }

    fn load_input_if_needed(&mut self) {
        if self.input_data.is_some() {
            return;
        }

        let Some(path) = &self.input_path else {
            return;
        };

        match load_input(path) {
            Ok(data) => {
                self.logs.push(format!(
                    "입력 로드 완료: avatar_bones={}, clothing_bones={}, clothing_smrs={}",
                    data.avatar_bones.len(),
                    data.clothing_bones.len(),
                    data.clothing_smrs.len()
                ));
                self.input_data = Some(data);
            }
            Err(err) => {
                self.logs.push(format!("입력 로드 실패: {}", err));
            }
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.actions.len();
    }

    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.actions.len() - 1
        } else {
            self.selected - 1
        };
    }

    fn run_selected(&mut self) {
        self.load_input_if_needed();
        let action = self.actions[self.selected];
        self.logs.push(format!("실행: {}", action.title()));

        let report = match &self.input_data {
            Some(data) => analyze(data),
            None => analyze(&sample_input()),
        };

        self.logs.push(format!(
            "중복(아바타): {}",
            report.duplicate_avatar_bone_names.len()
        ));
        self.logs.push(format!(
            "중복(의상): {}",
            report.duplicate_clothing_bone_names.len()
        ));
        self.logs.push(format!(
            "예상 이동 본/SMR: {}/{}",
            report.estimated_moved_bones, report.estimated_moved_smrs
        ));
        self.logs.push(format!(
            "예상 이름 변경 본/SMR: {}/{}",
            report.estimated_renamed_bones, report.estimated_renamed_smrs
        ));

        if !report.warnings.is_empty() {
            self.logs
                .push(format!("경고: {}", report.warnings.join(" | ")));
        }

        self.latest_report = Some(report);
        self.trim_logs();
    }

    fn save_report(&mut self) {
        let Some(report) = &self.latest_report else {
            self.logs
                .push("저장할 결과가 없습니다. 먼저 작업을 실행하세요.".to_string());
            self.trim_logs();
            return;
        };

        let Some(path) = &self.output_path else {
            self.logs
                .push("출력 경로가 없어 저장을 건너뜁니다.".to_string());
            self.trim_logs();
            return;
        };

        match save_report(path, report) {
            Ok(()) => self
                .logs
                .push(format!("결과 저장 완료: {}", path.display())),
            Err(err) => self.logs.push(format!("결과 저장 실패: {}", err)),
        }
        self.trim_logs();
    }

    fn trim_logs(&mut self) {
        if self.logs.len() > 120 {
            let drain = self.logs.len() - 120;
            self.logs.drain(0..drain);
        }
    }
}

fn main() -> Result<()> {
    let args = parse_args()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.input, args.output);
    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => app.run_selected(),
                    KeyCode::Char('s') => app.save_report(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(10),
        ])
        .split(frame.size());

    let status_text = format!(
        "VtuberVRMTool Rust TUI | q: 종료, ↑/↓: 이동, Enter: 실행, s: 저장 | 입력: {}",
        app.input_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "예시 데이터".to_string())
    );
    let title = Paragraph::new(Line::from(vec![
        Span::styled("상태 ", Style::default().fg(Color::Yellow)),
        Span::raw(status_text),
    ]))
    .block(Block::default().borders(Borders::ALL).title("VRM TUI"));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let prefix = if idx == app.selected { "> " } else { "  " };
            ListItem::new(Line::from(format!("{}{}", prefix, action.title())))
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("작업 목록"))
        .highlight_style(Style::default().fg(Color::Cyan));
    frame.render_widget(menu, chunks[1]);

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .take(20)
        .rev()
        .map(|log| Line::from(log.as_str()))
        .collect();

    let logs =
        Paragraph::new(log_lines).block(Block::default().borders(Borders::ALL).title("실행 로그"));
    frame.render_widget(logs, chunks[2]);
}

fn analyze(input: &VrmInput) -> VrmReport {
    let avatar_duplicates = duplicate_names(&input.avatar_bones);
    let clothing_duplicates = duplicate_names(&input.clothing_bones);

    let clothing_name_set: HashSet<&str> = input
        .clothing_bones
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    let avatar_name_set: HashSet<&str> =
        input.avatar_bones.iter().map(|b| b.name.as_str()).collect();

    let mut referenced = HashSet::new();
    for smr in &input.clothing_smrs {
        if let Some(root) = &smr.root_bone {
            if clothing_name_set.contains(root.as_str()) {
                referenced.insert(root.as_str());
            }
        }
        for bone in &smr.bones {
            if clothing_name_set.contains(bone.as_str()) {
                referenced.insert(bone.as_str());
            }
        }
    }

    let estimated_moved_bones = referenced
        .iter()
        .filter(|name| avatar_name_set.contains(**name))
        .count();

    let suffix = normalize_suffix(&input.suffix);
    let estimated_renamed_bones = if suffix.is_empty() {
        0
    } else {
        referenced
            .iter()
            .filter(|name| !name.ends_with(&suffix))
            .count()
    };
    let estimated_renamed_smrs = if suffix.is_empty() {
        0
    } else {
        input
            .clothing_smrs
            .iter()
            .filter(|smr| !smr.name.ends_with(&suffix))
            .count()
    };

    let mut warnings = Vec::new();
    if !avatar_duplicates.is_empty() {
        warnings.push("아바타에 동일한 본 이름이 있어 매칭이 모호할 수 있습니다.".to_string());
    }
    if !clothing_duplicates.is_empty() {
        warnings
            .push("의상 본 이름 중복으로 이동/이름변경 결과가 불안정할 수 있습니다.".to_string());
    }
    if input.clothing_smrs.is_empty() {
        warnings.push("의상 SMR이 없어 병합 이동 대상이 없습니다.".to_string());
    }

    VrmReport {
        duplicate_avatar_bone_names: avatar_duplicates,
        duplicate_clothing_bone_names: clothing_duplicates,
        referenced_clothing_bones: referenced.len(),
        estimated_moved_bones,
        estimated_moved_smrs: input.clothing_smrs.len(),
        estimated_renamed_bones,
        estimated_renamed_smrs,
        warnings,
    }
}

fn duplicate_names(bones: &[BoneInfo]) -> Vec<String> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for bone in bones {
        let entry = counts.entry(bone.name.as_str()).or_insert(0);
        *entry += 1;
    }

    let mut duplicates: Vec<String> = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name.to_string())
        .collect();
    duplicates.sort();
    duplicates
}

fn normalize_suffix(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('_');
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('_') {
        trimmed.to_string()
    } else {
        format!("_{}", trimmed)
    }
}

fn load_input(path: &Path) -> Result<VrmInput> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("입력 파일을 읽을 수 없습니다: {}", path.display()))?;
    let data: VrmInput = serde_json::from_str(&text)
        .with_context(|| format!("입력 JSON 파싱 실패: {}", path.display()))?;
    Ok(data)
}

fn save_report(path: &Path, report: &VrmReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("출력 폴더 생성 실패: {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(report).context("결과 JSON 직렬화 실패")?;
    fs::write(path, text).with_context(|| format!("결과 파일 쓰기 실패: {}", path.display()))?;
    Ok(())
}

#[derive(Default)]
struct Args {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = std::env::args().skip(1);
    let mut parsed = Args::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                let value = args.next().context("--input 뒤에 경로가 필요합니다.")?;
                parsed.input = Some(PathBuf::from(value));
            }
            "--output" => {
                let value = args.next().context("--output 뒤에 경로가 필요합니다.")?;
                parsed.output = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(anyhow::anyhow!(format!("알 수 없는 인자: {}", other)));
            }
        }
    }

    Ok(parsed)
}

fn print_help() {
    println!("vrm-tui 사용법");
    println!("  cargo run -- --input <path> --output <path>");
    println!("인자");
    println!("  --input   Unity에서 내보낸 VRM 입력 JSON 경로");
    println!("  --output  분석 결과 JSON 저장 경로");
}

fn sample_input() -> VrmInput {
    VrmInput {
        avatar_bones: vec![
            BoneInfo {
                name: "Hips".to_string(),
                parent_name: None,
            },
            BoneInfo {
                name: "Spine".to_string(),
                parent_name: Some("Hips".to_string()),
            },
        ],
        clothing_bones: vec![
            BoneInfo {
                name: "Spine".to_string(),
                parent_name: None,
            },
            BoneInfo {
                name: "Chest".to_string(),
                parent_name: Some("Spine".to_string()),
            },
        ],
        clothing_smrs: vec![SmrInfo {
            name: "Jacket".to_string(),
            root_bone: Some("Spine".to_string()),
            bones: vec!["Spine".to_string(), "Chest".to_string()],
        }],
        suffix: "_cloth".to_string(),
    }
}
