Based on the information available, the gitlab_ops repository is designed for testing GitLab GraphQL APIs. Below is a README that outlines the project’s purpose, setup instructions, and usage guidelines.

gitlab_ops

gitlab_ops is a tool for testing and interacting with GitLab’s GraphQL APIs. It provides a command-line interface (CLI) to fetch and display project information from a specified GitLab instance.

Features

	•	Fetches a list of projects from a GitLab instance using GraphQL.
	•	Displays project details, including name, description, and web URL.
	•	Interactive terminal user interface for navigating through projects.

Prerequisites

	•	Rust programming language installed.
	•	Access to a GitLab instance with appropriate API permissions.

Installation

	1.	Clone the repository:

git clone https://github.com/edgarhsanchez/gitlab_ops.git
cd gitlab_ops


	2.	Build the project:

cargo build --release



Configuration

The application requires a GitLab access token and the GitLab host URL. These can be provided through environment variables or will be prompted during runtime.
	1.	Using Environment Variables:
Create a .env file in the project directory with the following content:

GITLAB_TOKEN=your_access_token
GITLAB_HOST=gitlab.example.com


	2.	Runtime Input:
If the environment variables are not set, the application will prompt you to enter the GitLab token and host during execution.

Usage

Run the application using the following command:

cargo run --release

Use the arrow keys to navigate through the list of projects. Press Esc or q to exit the application.

License

This project is licensed under the MIT License.

This README provides an overview of the gitlab_ops project, including its purpose, setup instructions, and usage guidelines. For more detailed information, please refer to the source code and comments within the repository.
