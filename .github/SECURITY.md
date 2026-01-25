# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Soketi.rs seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to:
- **Email**: security@ferdiunal.com (or your security email)

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

- Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit the issue

This information will help us triage your report more quickly.

## Preferred Languages

We prefer all communications to be in English or Turkish.

## Security Update Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any potential similar problems
3. Prepare fixes for all supported releases
4. Release new security fix versions as soon as possible

## Comments on this Policy

If you have suggestions on how this process could be improved, please submit a pull request.

## Disclosure Policy

When we receive a security bug report, we will:

- Confirm the problem and determine affected versions
- Audit code to find any similar problems
- Prepare fixes for all supported versions
- Release patched versions
- Publicly disclose the vulnerability after fixes are released

## Security Best Practices

When deploying Soketi.rs in production:

1. **Use HTTPS/WSS**: Always use TLS encryption for WebSocket connections
2. **Secure Secrets**: Never commit secrets to version control
3. **Rate Limiting**: Configure appropriate rate limits
4. **Authentication**: Enable authentication for private and presence channels
5. **CORS**: Configure CORS to allow only trusted origins
6. **Updates**: Keep Soketi.rs and dependencies up to date
7. **Monitoring**: Enable metrics and monitoring
8. **Network Security**: Use firewalls and network policies
9. **Database Security**: Secure database connections with strong passwords
10. **Redis Security**: Enable Redis authentication and use secure connections

## Known Security Considerations

### WebSocket Security
- Always validate and sanitize user input
- Implement proper authentication for private channels
- Use rate limiting to prevent abuse

### API Security
- Validate all API requests
- Use HMAC signatures for webhook verification
- Implement proper CORS policies

### Infrastructure Security
- Keep Docker images updated
- Use non-root users in containers
- Secure Redis and database connections
- Use secrets management for sensitive data

## Security Hall of Fame

We would like to thank the following individuals for responsibly disclosing security issues:

<!-- Add names here as security issues are reported and fixed -->

## Contact

For any security-related questions or concerns, please contact:
- **Email**: security@ferdiunal.com
- **GitHub**: [@ferdiunal](https://github.com/ferdiunal)
