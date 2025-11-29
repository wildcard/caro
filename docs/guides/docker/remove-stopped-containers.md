---
id: "guide-docker-001"
title: "Remove all stopped Docker containers"
description: "Clean up disk space by removing all stopped containers at once"
category: Docker
difficulty: Beginner
tags: [docker, cleanup, containers, prune, disk-space]
natural_language_prompt: "remove all stopped docker containers"
generated_command: "docker container prune -f"
shell_type: Bash
risk_level: Moderate
author: "cmdai-community"
created_at: "2024-01-15T11:00:00Z"
updated_at: "2024-11-28T15:00:00Z"
prerequisites:
  - "Docker is installed and running"
  - "You have stopped containers to remove"
expected_outcomes:
  - "All stopped containers are removed"
  - "Disk space is freed"
  - "Running containers are unaffected"
related_guides:
  - "guide-docker-002"
  - "guide-docker-010"
related_guardrails: []
alternatives:
  - "docker container prune          # Interactive confirmation"
  - "docker rm $(docker ps -aq)      # Remove all containers (⚠️ including running)"
  - "docker system prune -a          # Remove everything unused (⚠️ aggressive)"
---

# Remove All Stopped Docker Containers

## What it does

Removes all Docker containers that are in the "Exited" or "Created" state, freeing up disk space. Running containers are not affected.

## When to use this

- ✅ Your disk is running low on space
- ✅ You have many old test containers cluttering your system
- ✅ You want to clean up after development work
- ✅ You're preparing for a fresh start
- ⚠️ **Be careful** if stopped containers have data you need
- ⚠️ **Don't use** if you plan to restart stopped containers

## The cmdai way

```bash
cmdai "remove all stopped docker containers"
```

cmdai generates:
```bash
docker container prune -f
```

## Understanding the command

```bash
docker container prune -f
```

Breaking it down:
- `docker container`: Docker container management commands
- `prune`: Remove unused containers
- `-f` or `--force`: Skip confirmation prompt (non-interactive)

**Without -f flag:**
```bash
docker container prune
WARNING! This will remove all stopped containers.
Are you sure you want to continue? [y/N]
```

## Step-by-step example

Check your containers:
```bash
$ docker ps -a
CONTAINER ID   IMAGE          STATUS                     NAMES
abc123def456   nginx:latest   Up 2 hours                 web-server
ghi789jkl012   redis:7        Exited (0) 3 days ago     cache
mno345pqr678   postgres:15    Exited (137) 1 week ago   database
stu901vwx234   python:3.11    Exited (0) 2 hours ago    test-script
```

Run the prune:
```bash
$ docker container prune -f
Deleted Containers:
ghi789jkl012
mno345pqr678
stu901vwx234

Total reclaimed space: 1.2GB
```

Verify:
```bash
$ docker ps -a
CONTAINER ID   IMAGE          STATUS        NAMES
abc123def456   nginx:latest   Up 2 hours    web-server
```

Only running containers remain!

## Safety notes

⚠️ **Moderate risk** - Stopped containers are permanently deleted

✓ **Preserves running** - Running containers are never affected

✗ **No undo** - Once deleted, you'll need to recreate containers

⚠️ **Volume data** - Container data is lost unless stored in volumes

## What gets deleted vs. preserved

**Deleted:**
- ✗ Stopped containers (Exited status)
- ✗ Container filesystem changes
- ✗ Container-specific configuration

**Preserved:**
- ✓ Running containers
- ✓ Docker images (can recreate containers)
- ✓ Docker volumes (persistent data)
- ✓ Networks

## Before you prune: Save important data

If a stopped container has data you need:

```bash
# Copy files from stopped container
docker cp container_name:/path/to/file ./backup/

# Or commit container to image
docker commit container_name my-backup-image

# Or export container filesystem
docker export container_name > container-backup.tar
```

## Less destructive alternatives

**Remove specific container:**
```bash
docker rm container_name
```

**Interactive prune (asks for confirmation):**
```bash
docker container prune
# Answer 'y' to confirm
```

**Filter what to prune:**
```bash
# Only containers stopped more than 24 hours ago
docker container prune --filter "until=24h"

# Only containers with specific label
docker container prune --filter "label=temporary"
```

## More aggressive cleanup

**Remove ALL containers (including running!):**
```bash
# ⚠️ DANGEROUS - Stops and removes everything
docker rm -f $(docker ps -aq)
```

**Full system cleanup:**
```bash
# ⚠️ AGGRESSIVE - Removes images, volumes, networks too
docker system prune -a --volumes
```

## Common use cases

**Daily development cleanup:**
```bash
# At end of day, clean up test containers
docker container prune -f
```

**CI/CD pipeline cleanup:**
```bash
# After tests run, clean up
docker container prune -f
docker image prune -f
```

**Emergency disk space recovery:**
```bash
# Nuclear option - clean everything unused
docker system prune -a -f
```

## Related guides

- [Remove all Docker images](../docker/remove-images.md)
- [Clean up Docker volumes](../docker/prune-volumes.md)
- [Complete Docker system cleanup](../docker/system-prune.md)
- [Remove specific Docker container](../docker/remove-container.md)

## Try it yourself

```bash
# Check what would be removed first
docker container ls -a --filter "status=exited"

# Try this guide in cmdai
cmdai guides run guide-docker-001

# Or execute the command directly
docker container prune -f

# Verify results
docker ps -a
```

## Community metrics

- **Upvotes:** 89
- **Downvotes:** 4
- **Execution count:** 2,145
- **Success rate:** 96%
- **Quality score:** 0.91

## Community feedback

> "Use this every Friday to clean up my dev environment. Works perfectly!" - *devops_daily*

> "Saved me 50GB of disk space. Didn't realize I had so many stopped containers!" - *disk_space_crisis*

> "Add warning about volume data preservation - almost lost important database files" - *data_engineer*
> *Note: Added volume preservation section - thanks for feedback!*
