# GitHub Repository Settings

This document contains the recommended settings for the Soketi.rs GitHub repository.

## Repository Description

```
🚀 High-performance, Pusher-compatible WebSocket server written in Rust. Real-time messaging with public, private & presence channels. Docker-ready with Redis clustering support.
```

## Topics (Keywords)

Add these topics to help people discover your repository:

```
websocket
pusher
real-time
rust
tokio
redis
docker
websocket-server
pusher-protocol
presence-channels
private-channels
real-time-messaging
horizontal-scaling
prometheus-metrics
high-performance
```

## About Section

### Website
```
https://github.com/ferdiunal/soketi.rs
```

### Social Preview Image
Upload a social preview image (1280x640px) showing:
- Soketi.rs logo
- Key features
- Performance metrics

## Repository Settings

### General

- ✅ **Issues**: Enabled
- ✅ **Projects**: Enabled (optional)
- ✅ **Wiki**: Disabled (use docs/ instead)
- ✅ **Discussions**: Enabled
- ✅ **Sponsorships**: Enabled (via FUNDING.yml)

### Features to Enable

1. **GitHub Pages**
   - Source: `main` branch
   - Folder: `/docs`
   - Custom domain: (optional)
   - Enforce HTTPS: Yes

2. **Discussions**
   - Categories:
     - 💬 General
     - 💡 Ideas
     - 🙏 Q&A
     - 📣 Announcements
     - 🐛 Bug Reports
     - 🎉 Show and Tell

3. **Security**
   - ✅ Dependency graph: Enabled
   - ✅ Dependabot alerts: Enabled
   - ✅ Dependabot security updates: Enabled
   - ✅ Code scanning: Enabled (GitHub Advanced Security)
   - ✅ Secret scanning: Enabled

4. **Branch Protection (main)**
   - ✅ Require pull request reviews before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Include administrators
   - ✅ Restrict who can push to matching branches

### Labels

Create these labels for better issue organization:

**Type**
- `bug` - Something isn't working (red)
- `enhancement` - New feature or request (blue)
- `documentation` - Improvements or additions to documentation (green)
- `performance` - Performance improvements (orange)
- `security` - Security-related issues (red)

**Priority**
- `priority: critical` - Critical priority (red)
- `priority: high` - High priority (orange)
- `priority: medium` - Medium priority (yellow)
- `priority: low` - Low priority (green)

**Status**
- `status: needs-triage` - Needs triage (gray)
- `status: in-progress` - Work in progress (yellow)
- `status: blocked` - Blocked by something (red)
- `status: ready` - Ready for implementation (green)

**Component**
- `component: adapter` - Adapter-related (blue)
- `component: cache` - Cache-related (blue)
- `component: database` - Database-related (blue)
- `component: docker` - Docker-related (blue)
- `component: metrics` - Metrics-related (blue)
- `component: websocket` - WebSocket-related (blue)

**Good First Issue**
- `good first issue` - Good for newcomers (purple)
- `help wanted` - Extra attention is needed (green)

## GitHub Actions Secrets

Add these secrets for CI/CD:

```
DOCKER_HUB_USERNAME=ferdiunal
DOCKER_HUB_TOKEN=<your-token>
CARGO_REGISTRY_TOKEN=<your-token>  # For crates.io publishing
```

## Social Links

Add these to your GitHub profile and README:

- **GitHub**: https://github.com/ferdiunal/soketi.rs
- **Docker Hub**: https://hub.docker.com/r/funal/soketi-rs
- **Crates.io**: https://crates.io/crates/soketi-rs (when published)
- **Documentation**: https://github.com/ferdiunal/soketi.rs/tree/main/docs

## README Badges

Current badges in README.md:
- ✅ Rust version
- ✅ Build status
- ✅ Version
- ✅ License
- ✅ Docker pulls
- ✅ Docker image size

Consider adding:
- Crates.io version (when published)
- Documentation status
- Code coverage
- Security audit status

## Community Health Files

Created files:
- ✅ `.github/CONTRIBUTING.md` - Contribution guidelines
- ✅ `.github/SECURITY.md` - Security policy
- ✅ `.github/FUNDING.yml` - Sponsor button
- ✅ `.github/PULL_REQUEST_TEMPLATE.md` - PR template
- ✅ `.github/ISSUE_TEMPLATE/` - Issue templates
- ✅ `LICENSE` - MIT License (already exists)
- ✅ `CODE_OF_CONDUCT.md` - (create if needed)

## Recommended Actions

1. **Enable GitHub Pages**
   ```
   Settings → Pages → Source: main branch → /docs folder
   ```

2. **Enable Discussions**
   ```
   Settings → Features → Discussions → Enable
   ```

3. **Add Topics**
   ```
   Repository main page → About → Topics → Add topics
   ```

4. **Update Description**
   ```
   Repository main page → About → Description → Edit
   ```

5. **Add Social Preview**
   ```
   Settings → Social preview → Upload image
   ```

6. **Configure Branch Protection**
   ```
   Settings → Branches → Add rule → main
   ```

7. **Enable Security Features**
   ```
   Settings → Security & analysis → Enable all
   ```

## GitHub Pages Setup

If you want to host documentation on GitHub Pages:

1. Create `docs/index.html` or use Jekyll
2. Enable GitHub Pages in settings
3. Choose `/docs` folder as source
4. Access at: `https://ferdiunal.github.io/soketi.rs/`

## Maintenance

Regular tasks:
- Review and merge Dependabot PRs
- Respond to issues within 48 hours
- Update documentation as features are added
- Release new versions regularly
- Keep Docker images updated
- Monitor security alerts

## Support Channels

- **Issues**: Bug reports and feature requests
- **Discussions**: Questions and community chat
- **Email**: For security issues only
- **Twitter/X**: Announcements (optional)
