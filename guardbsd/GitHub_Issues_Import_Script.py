#!/usr/bin/env python3
"""
GuardBSD GitHub Issues Import Script
Automatically creates all issues, labels, and milestones from issues.md
"""

import os
import sys
import json
import time
import re
from typing import List, Dict, Optional
from datetime import datetime, timedelta
import requests

# Configuration
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")
REPO_OWNER = "cartesian-school"  # Change to your GitHub username/org
REPO_NAME = "guardbsd"
DRY_RUN = False  # Set to True to test without creating issues

# GitHub API base URL
API_BASE = "https://api.github.com"
REPO_API = f"{API_BASE}/repos/{REPO_OWNER}/{REPO_NAME}"

# Rate limiting
RATE_LIMIT_DELAY = 1  # seconds between API calls


class GitHubImporter:
    def __init__(self, token: str, owner: str, repo: str, dry_run: bool = False):
        self.token = token
        self.owner = owner
        self.repo = repo
        self.dry_run = dry_run
        self.headers = {
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
        }
        self.label_cache = {}
        self.milestone_cache = {}

    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None) -> Dict:
        """Make authenticated request to GitHub API"""
        url = f"{REPO_API}/{endpoint}"
        
        if self.dry_run:
            print(f"[DRY RUN] {method} {url}")
            if data:
                print(f"[DRY RUN] Data: {json.dumps(data, indent=2)}")
            return {"dry_run": True}
        
        response = requests.request(method, url, headers=self.headers, json=data)
        
        if response.status_code not in [200, 201]:
            print(f"Error: {response.status_code} - {response.text}")
            return None
        
        time.sleep(RATE_LIMIT_DELAY)
        return response.json()

    def create_label(self, name: str, color: str, description: str = "") -> bool:
        """Create a label if it doesn't exist"""
        if name in self.label_cache:
            return True
        
        # Check if label exists
        response = self._request("GET", f"labels/{name}")
        if response and not response.get("dry_run"):
            self.label_cache[name] = True
            return True
        
        # Create label
        data = {
            "name": name,
            "color": color,
            "description": description
        }
        response = self._request("POST", "labels", data)
        
        if response:
            self.label_cache[name] = True
            print(f"✓ Created label: {name}")
            return True
        
        return False

    def create_milestone(self, title: str, due_date: Optional[str] = None, description: str = "") -> Optional[int]:
        """Create a milestone if it doesn't exist"""
        if title in self.milestone_cache:
            return self.milestone_cache[title]
        
        # Check if milestone exists
        response = self._request("GET", "milestones")
        if response and not response.get("dry_run"):
            for milestone in response:
                if milestone["title"] == title:
                    self.milestone_cache[title] = milestone["number"]
                    return milestone["number"]
        
        # Create milestone
        data = {
            "title": title,
            "description": description,
            "state": "open"
        }
        
        if due_date:
            data["due_on"] = due_date
        
        response = self._request("POST", "milestones", data)
        
        if response and not response.get("dry_run"):
            milestone_number = response["number"]
            self.milestone_cache[title] = milestone_number
            print(f"✓ Created milestone: {title}")
            return milestone_number
        
        return None

    def create_issue(self, title: str, body: str, labels: List[str], 
                    milestone: Optional[str] = None, assignees: List[str] = None) -> bool:
        """Create an issue"""
        data = {
            "title": title,
            "body": body,
            "labels": labels
        }
        
        if milestone and milestone in self.milestone_cache:
            data["milestone"] = self.milestone_cache[milestone]
        
        if assignees:
            data["assignees"] = assignees
        
        response = self._request("POST", "issues", data)
        
        if response:
            if not response.get("dry_run"):
                print(f"✓ Created issue: #{response['number']} - {title}")
            else:
                print(f"✓ [DRY RUN] Would create issue: {title}")
            return True
        
        return False


def parse_issues_from_markdown(filename: str) -> Dict:
    """Parse issues from markdown file"""
    with open(filename, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract milestones
    milestones = []
    milestone_pattern = r'### Milestone: (v[\d.]+-[\w-]+) \((Q\d 20\d{2})\)(.*?)(?=###|\Z)'
    for match in re.finditer(milestone_pattern, content, re.DOTALL):
        milestone_name = match.group(1)
        quarter = match.group(2)
        description = match.group(3).strip()
        
        # Calculate due date from quarter
        year = int(quarter.split()[-1])
        q = int(quarter[1])
        month = (q * 3)
        due_date = f"{year}-{month:02d}-01T00:00:00Z"
        
        milestones.append({
            "title": milestone_name,
            "due_date": due_date,
            "description": description[:200]  # GitHub limit
        })
    
    # Extract issues
    issues = []
    issue_pattern = r'### Issue #(\d+): (.+?)\n\*\*Labels:\*\* `(.+?)`.*?\*\*Milestone:\*\* ([\w.-]+).*?\*\*Estimate:\*\* (.+?)\n.*?\*\*Description:\*\*\n(.+?)\*\*Tasks:\*\*(.+?)(?:\*\*Acceptance Criteria:\*\*(.+?))?(?:\*\*Dependencies:\*\*(.+?))?(?=###|\Z)'
    
    for match in re.finditer(issue_pattern, content, re.DOTALL):
        issue_number = int(match.group(1))
        title = match.group(2).strip()
        labels = [l.strip() for l in match.group(3).split(',')]
        milestone = match.group(4).strip()
        estimate = match.group(5).strip()
        description = match.group(6).strip()
        tasks = match.group(7).strip()
        acceptance = match.group(8).strip() if match.group(8) else ""
        dependencies = match.group(9).strip() if match.group(9) else ""
        
        # Build issue body
        body = f"{description}\n\n"
        body += f"**Estimate:** {estimate}\n\n"
        body += f"## Tasks\n{tasks}\n\n"
        
        if acceptance:
            body += f"## Acceptance Criteria\n{acceptance}\n\n"
        
        if dependencies:
            body += f"## Dependencies\n{dependencies}\n\n"
        
        issues.append({
            "number": issue_number,
            "title": title,
            "body": body,
            "labels": labels,
            "milestone": milestone
        })
    
    return {
        "milestones": milestones,
        "issues": sorted(issues, key=lambda x: x["number"])
    }


def setup_labels(importer: GitHubImporter):
    """Create all necessary labels"""
    labels = {
        # Priority
        "priority-critical": ("d73a4a", "Critical priority - must have"),
        "priority-high": ("ff9800", "High priority - should have"),
        "priority-medium": ("ffc107", "Medium priority - nice to have"),
        "priority-low": ("8bc34a", "Low priority - future work"),
        
        # Component
        "microkernel": ("0052cc", "Microkernel code"),
        "uk-space": ("0066ff", "µK-Space (memory management)"),
        "uk-time": ("3399ff", "µK-Time (scheduler)"),
        "uk-ipc": ("66ccff", "µK-IPC (communication)"),
        "userland": ("00bcd4", "User-space programs"),
        "server": ("26c6da", "System servers"),
        "driver": ("4dd0e1", "Device drivers"),
        "infrastructure": ("546e7a", "Build/CI/tooling"),
        "documentation": ("9e9e9e", "Documentation"),
        "testing": ("795548", "Tests and QA"),
        "security": ("d32f2f", "Security-related"),
        "library": ("7b1fa2", "Library code"),
        
        # Architecture
        "x86_64": ("673ab7", "x86-64 specific"),
        "aarch64": ("512da8", "ARM64 specific"),
        "architecture": ("9c27b0", "Multi-architecture"),
        "riscv": ("8e24aa", "RISC-V specific"),
        
        # Phase
        "phase-1": ("1b5e20", "Foundation (weeks 1-16)"),
        "phase-2": ("2e7d32", "Core features (weeks 17-28)"),
        "phase-3": ("43a047", "Production ready (weeks 29-48)"),
        "phase-4": ("66bb6a", "Advanced features (2027+)"),
        
        # Type
        "enhancement": ("84b7c2", "New feature"),
        "bug": ("d73a4a", "Bug report"),
        "design": ("c5def5", "Design discussion"),
        "research": ("b39ddb", "Research task"),
        
        # Specific areas
        "boot": ("ff6f00", "Boot and initialization"),
        "api": ("ff9100", "API design"),
        "network": ("00acc1", "Networking"),
        "filesystem": ("8d6e63", "File systems"),
        "graphics": ("e91e63", "Graphics and GPU"),
        "power": ("fdd835", "Power management"),
        "real-time": ("c62828", "Real-time features"),
        "container": ("5e35b1", "Container support"),
        "tooling": ("616161", "Developer tools"),
        "platform": ("6d4c41", "Platform support"),
    }
    
    print("\n=== Creating Labels ===")
    for name, (color, description) in labels.items():
        importer.create_label(name, color, description)


def main():
    if not GITHUB_TOKEN:
        print("Error: GITHUB_TOKEN environment variable not set")
        print("Usage: export GITHUB_TOKEN=ghp_your_token_here")
        sys.exit(1)
    
    print(f"""
╔══════════════════════════════════════════════════════════════╗
║         GuardBSD GitHub Issues Import Script                 ║
║                                                              ║
║  Repository: {REPO_OWNER}/{REPO_NAME:<44} ║
║  Dry Run: {str(DRY_RUN):<50} ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    # Initialize importer
    importer = GitHubImporter(GITHUB_TOKEN, REPO_OWNER, REPO_NAME, DRY_RUN)
    
    # Setup labels
    setup_labels(importer)
    
    # Parse issues file
    print("\n=== Parsing issues.md ===")
    try:
        data = parse_issues_from_markdown("issues.md")
    except FileNotFoundError:
        print("Error: issues.md not found in current directory")
        sys.exit(1)
    
    print(f"Found {len(data['milestones'])} milestones and {len(data['issues'])} issues")
    
    # Create milestones
    print("\n=== Creating Milestones ===")
    for milestone in data["milestones"]:
        importer.create_milestone(
            milestone["title"],
            milestone["due_date"],
            milestone["description"]
        )
    
    # Create issues
    print("\n=== Creating Issues ===")
    created = 0
    failed = 0
    
    for issue in data["issues"]:
        success = importer.create_issue(
            issue["title"],
            issue["body"],
            issue["labels"],
            issue["milestone"]
        )
        
        if success:
            created += 1
        else:
            failed += 1
    
    # Summary
    print(f"""
╔══════════════════════════════════════════════════════════════╗
║                      Import Summary                          ║
╠══════════════════════════════════════════════════════════════╣
║  Milestones created: {len(data['milestones']):<42} ║
║  Issues created:     {created:<42} ║
║  Issues failed:      {failed:<42} ║
╚══════════════════════════════════════════════════════════════╝
""")
    
    if not DRY_RUN:
        print(f"\n✓ Done! Visit https://github.com/{REPO_OWNER}/{REPO_NAME}/issues")
    else:
        print(f"\n✓ Dry run complete. Set DRY_RUN=False to actually create issues.")


if __name__ == "__main__":
    main()
