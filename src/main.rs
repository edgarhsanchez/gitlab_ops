use dotenv::dotenv;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use reqwest::Client;
use serde_json::json;
use std::{
    env,
    io::{self, Write},
};
use std::fs::OpenOptions;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Project structure
#[derive(Debug, Clone)]
struct Project {
    name: String,
    description: String,
    web_url: String,
}

// Application state
struct AppState {
    projects: Vec<Project>,
    selected_index: usize,
    list_state: ListState,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Prompt for GitLab token
    let gitlab_token = get_gitlab_token().unwrap_or_else(|| {
        eprintln!("GitLab token is required. Exiting.");
        std::process::exit(1);
    });

    // Prompt for GitLab host
    let gitlab_host = get_gitlab_host().unwrap_or_else(|| {
        eprintln!("GitLab host is required. Exiting.");
        std::process::exit(1);
    });


    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Fetch projects from GitLab
    let projects = fetch_projects(gitlab_token, gitlab_host).unwrap_or_else(|err| {
        eprintln!("Failed to fetch projects: {}", err);
        vec![]
    });

    let mut app_state = AppState {
        projects,
        selected_index: 0,
        list_state: ListState::default(),
    };
    app_state.list_state.select(Some(0));

    // Main event loop
    let res = run_app(&mut terminal, &mut app_state);

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

// Run the main application loop
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app_state: &mut AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render_ui::<B>(frame, app_state))?;

        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Esc => {
                    //quit the program
                    return Ok(());
                },
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => {
                    if app_state.selected_index < app_state.projects.len() - 1 {
                        app_state.selected_index += 1;
                        app_state.list_state.select(Some(app_state.selected_index));
                    }
                }
                KeyCode::Up => {
                    if app_state.selected_index > 0 {
                        app_state.selected_index -= 1;
                        app_state.list_state.select(Some(app_state.selected_index));
                    }
                }
                _ => {}
            }
        }
    }
}

// Render the terminal UI
fn render_ui<B: Backend>(frame: &mut ratatui::Frame, app_state: &mut AppState) {
    let size = frame.area();

    // Split the layout into two sections: list and details
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    // List of projects
    let items: Vec<ListItem> = app_state
        .projects
        .iter()
        .map(|p| ListItem::new(p.name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Projects\\Esc to quit"))
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], &mut app_state.list_state);

    // Selected project details
    if let Some(project) = app_state.projects.get(app_state.selected_index) {
        let details = format!(
            "Name: {}\nDescription: {}\nWeb URL: {}",
            project.name,
            project.description,
            project.web_url
        );
        let paragraph = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Details"));
        frame.render_widget(paragraph, chunks[1]);
    }
}

// Fetch projects using the GitLab GraphQL API
fn fetch_projects(
    gitlab_token: String,
    gitlab_host: String,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    let query = r#"
        query Projects($first: Int) {
            projects(first: $first) {
                nodes {
                    id
                    name
                    description
                    webUrl
                }
            }
        }
    "#;

    let variables = json!({ "first": 100 });
    let payload = json!({
        "query": query,
        "variables": variables,
    });

    let client = Client::new();
    let response = tokio::runtime::Runtime::new()?.block_on(async {
        client
            .post(format!("https://{}/api/graphql", gitlab_host))
            .bearer_auth(gitlab_token)
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await
    })?;

    let mut projects = Vec::new();
    if let Some(nodes) = response
        .get("data")
        .and_then(|data| data.get("projects"))
        .and_then(|projects| projects.get("nodes"))
        .and_then(|nodes| nodes.as_array())
    {
        for node in nodes {
            let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("N/A").to_string();
            let description = node
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("No description")
                .to_string();
            let web_url = node.get("webUrl").and_then(|w| w.as_str()).unwrap_or("N/A").to_string();
            projects.push(Project {
                name,
                description,
                web_url,
            });
        }
    }

    Ok(projects)
}

// Retrieve or prompt for the GitLab token
fn get_gitlab_token() -> Option<String> {
    match env::var("GITLAB_TOKEN") {
        Ok(token) if !token.is_empty() => Some(token),
        _ => {
            println!("Enter your GitLab token (leave blank to exit): ");
            let mut token = String::new();
            io::stdin().read_line(&mut token).unwrap();
            let token = token.trim().to_string();
            if token.is_empty() {
                None // Return None if the user leaves it blank
            } else {
                if let Err(e) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(".env")
                    .and_then(|mut file| writeln!(file, "GITLAB_TOKEN={}", token))
                {
                    eprintln!("Failed to save token to .env: {}", e);
                }
                Some(token)
            }
        }
    }
}

// Retrieve or prompt for the GitLab host
fn get_gitlab_host() -> Option<String> {
    match env::var("GITLAB_HOST") {
        Ok(host) if !host.is_empty() => Some(host),
        _ => {
            println!("Enter your GitLab host (leave blank to exit): ");
            let mut host = String::new();
            io::stdin().read_line(&mut host).unwrap();
            let host = host.trim().to_string();
            if host.is_empty() {
                None // Return None if the user leaves it blank
            } else {
                if let Err(e) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(".env")
                    .and_then(|mut file| writeln!(file, "GITLAB_HOST={}", host))
                {
                    eprintln!("Failed to save host to .env: {}", e);
                }
                Some(host)
            }
        }
    }
}