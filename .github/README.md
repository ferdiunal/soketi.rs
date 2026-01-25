# GitHub Community Files

This directory contains GitHub-specific community health files and templates.

## Files Overview

### Community Health Files

- **CONTRIBUTING.md** - Guidelines for contributing to the project
- **SECURITY.md** - Security policy and vulnerability reporting
- **CODE_OF_CONDUCT.md** - Community code of conduct (in root)
- **FUNDING.yml** - GitHub Sponsors configuration

### Issue Templates

Located in `ISSUE_TEMPLATE/`:
- **bug_report.md** - Template for bug reports
- **feature_request.md** - Template for feature requests
- **config.yml** - Issue template configuration with links

### Pull Request Template

- **PULL_REQUEST_TEMPLATE.md** - Template for pull requests

### Workflows

Located in `workflows/`:
- **docker-publish.yml** - Automated Docker image building and publishing

### Documentation

- **REPOSITORY_SETTINGS.md** - Recommended GitHub repository settings
- **DOCKER_SETUP.md** - Docker setup documentation

## Setting Up Your Repository

Follow the instructions in `REPOSITORY_SETTINGS.md` to:

1. Configure repository description and topics
2. Enable GitHub features (Discussions, Pages, etc.)
3. Set up branch protection rules
4. Configure security features
5. Add labels for issue organization

## GitHub Pages

The repository is configured to use GitHub Pages with the `/docs` folder.

Access documentation at: `https://ferdiunal.github.io/soketi.rs/`

## Sponsor Button

The sponsor button is configured in `FUNDING.yml`. Update with your sponsor links:
- GitHub Sponsors
- Patreon
- Ko-fi
- Open Collective
- Custom links

## Automated Workflows

### Docker Publishing

The `docker-publish.yml` workflow automatically:
- Builds Docker images on push to main
- Publishes to Docker Hub
- Creates multi-platform images (amd64, arm64)
- Tags images with version numbers

## Maintenance

Regular tasks:
- Review and update issue templates
- Monitor security alerts
- Update workflows as needed
- Keep documentation current

## Support

For questions about these files:
- Check [REPOSITORY_SETTINGS.md](REPOSITORY_SETTINGS.md)
- Open a [Discussion](https://github.com/ferdiunal/soketi.rs/discussions)
- Contact [@ferdiunal](https://github.com/ferdiunal)
