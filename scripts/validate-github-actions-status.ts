#!/usr/bin/env -S deno run --allow-net --allow-env

/**
 * Validate and report GitHub Actions workflow status
 * Fetches information about the latest workflow runs and their job statuses
 */

interface WorkflowRun {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  created_at: string;
  html_url: string;
  head_commit: {
    message: string;
  };
}

interface Job {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  html_url: string;
  steps?: Array<{
    name: string;
    status: string;
    conclusion: string | null;
  }>;
}

interface WorkflowRunsResponse {
  workflow_runs: WorkflowRun[];
}

interface JobsResponse {
  jobs: Job[];
}

const REPO_OWNER = "paiml";
const REPO_NAME = "paiml-mcp-agent-toolkit";
const GITHUB_API_BASE = "https://api.github.com";

async function fetchGitHubAPI(endpoint: string): Promise<Response> {
  const url = `${GITHUB_API_BASE}${endpoint}`;
  const headers: Record<string, string> = {
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "paiml-mcp-agent-toolkit",
  };

  // Use GitHub token if available for higher rate limits
  const token = Deno.env.get("GITHUB_TOKEN");
  if (token) {
    headers["Authorization"] = `token ${token}`;
  }

  const response = await fetch(url, { headers });

  if (!response.ok) {
    throw new Error(
      `GitHub API error: ${response.status} ${response.statusText}`,
    );
  }

  return response;
}

async function getLatestWorkflowRuns(): Promise<WorkflowRun[]> {
  const response = await fetchGitHubAPI(
    `/repos/${REPO_OWNER}/${REPO_NAME}/actions/runs?per_page=10`,
  );
  const data: WorkflowRunsResponse = await response.json();
  return data.workflow_runs;
}

async function getWorkflowJobs(runId: number): Promise<Job[]> {
  const response = await fetchGitHubAPI(
    `/repos/${REPO_OWNER}/${REPO_NAME}/actions/runs/${runId}/jobs`,
  );
  const data: JobsResponse = await response.json();
  return data.jobs;
}

function formatStatus(status: string, conclusion: string | null): string {
  const statusEmoji = {
    completed: conclusion === "success"
      ? "✅"
      : conclusion === "failure"
      ? "❌"
      : "⚠️",
    in_progress: "🔄",
    queued: "⏳",
  };

  const emoji = statusEmoji[status as keyof typeof statusEmoji] || "❓";
  const displayStatus = conclusion || status;

  return `${emoji} ${displayStatus}`;
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString();
}

async function validateGitHubActionsStatus() {
  console.log(
    "🔍 Fetching GitHub Actions status for paiml/paiml-mcp-agent-toolkit\n",
  );

  try {
    // Get latest workflow runs
    const runs = await getLatestWorkflowRuns();

    if (runs.length === 0) {
      console.log("No workflow runs found.");
      return;
    }

    // Group runs by workflow name
    const workflowGroups = new Map<string, WorkflowRun[]>();
    for (const run of runs) {
      const group = workflowGroups.get(run.name) || [];
      group.push(run);
      workflowGroups.set(run.name, group);
    }

    // Display status for each workflow
    console.log("📊 Latest Workflow Status:\n");

    let hasFailures = false;
    const failedJobs: Array<{ workflow: string; job: Job; run: WorkflowRun }> =
      [];

    for (const [workflowName, workflowRuns] of workflowGroups) {
      const latestRun = workflowRuns[0]; // Already sorted by created_at

      console.log(
        `${
          formatStatus(latestRun.status, latestRun.conclusion)
        } ${workflowName}`,
      );
      console.log(`   📅 ${formatDate(latestRun.created_at)}`);
      console.log(`   💬 ${latestRun.head_commit.message.split("\n")[0]}`);
      console.log(`   🔗 ${latestRun.html_url}`);

      // If the workflow failed, get job details
      if (latestRun.conclusion === "failure") {
        hasFailures = true;
        const jobs = await getWorkflowJobs(latestRun.id);

        console.log("\n   Failed Jobs:");
        for (const job of jobs) {
          if (job.conclusion === "failure") {
            failedJobs.push({ workflow: workflowName, job, run: latestRun });
            console.log(
              `   ${formatStatus(job.status, job.conclusion)} ${job.name}`,
            );
            console.log(`      🔗 ${job.html_url}`);
          }
        }
      }
      console.log();
    }

    // Detailed failure analysis
    if (failedJobs.length > 0) {
      console.log("\n❌ Failure Analysis:\n");

      for (const { workflow, job } of failedJobs) {
        console.log(`${workflow} → ${job.name}`);

        // Try to get more details about the failure
        try {
          const jobDetails = await fetchGitHubAPI(
            `/repos/${REPO_OWNER}/${REPO_NAME}/actions/jobs/${job.id}`,
          );
          const jobData = await jobDetails.json();

          if (jobData.steps) {
            console.log("   Failed steps:");
            for (const step of jobData.steps) {
              if (step.conclusion === "failure") {
                console.log(`   - ${step.name}`);
              }
            }
          }
        } catch (error) {
          console.log(`   Could not fetch detailed job information: ${error}`);
        }
        console.log();
      }

      console.log("\n💡 Common Fixes:");
      console.log(
        "   - CI failures: Check for build, test, or coverage issues",
      );
      console.log(
        "   - Code Quality failures: Check coverage threshold (70%) or complexity tools",
      );
      console.log(
        "   - PR Checks failures: Verify PR has proper labels and descriptions",
      );
      console.log(
        "   - Dependencies failures: Update or pin problematic dependencies",
      );
      console.log(
        "   - Security Audit failures: Run 'cargo audit fix' or update dependencies",
      );
      console.log(
        "   - Benchmark failures: Check benchmark thresholds and performance",
      );
    }

    // Summary
    console.log("\n📈 Summary:");
    console.log(`   Total workflows checked: ${workflowGroups.size}`);
    console.log(
      `   Status: ${
        hasFailures
          ? "❌ Some workflows are failing"
          : "✅ All workflows passing"
      }`,
    );

    if (hasFailures) {
      Deno.exit(1);
    }
  } catch (error) {
    console.error("❌ Error fetching GitHub Actions status:", error);
    console.error("\n💡 Tips:");
    console.error("   - Check your internet connection");
    console.error("   - Verify the repository exists and is public");
    console.error(
      "   - Set GITHUB_TOKEN environment variable for higher rate limits",
    );
    console.error(
      "   - Rate limit: 60 requests/hour without token, 5000 with token",
    );
    Deno.exit(1);
  }
}

// Run the validation
if (import.meta.main) {
  await validateGitHubActionsStatus();
}
