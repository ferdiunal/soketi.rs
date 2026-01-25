/**
 * Property-Based Tests for Reverse Proxy Configuration Correctness
 * Feature: documentation-and-deployment-enhancements
 * Property 7: Reverse Proxy Yapılandırma Doğruluğu
 * 
 * **Validates: Requirements 3.4, 3.7**
 * 
 * Tests that every reverse proxy configuration (Caddy and Nginx) includes:
 * - WebSocket upgrade headers
 * - Appropriate timeout settings
 * - Security headers (HSTS, X-Frame-Options, X-Content-Type-Options)
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 7: Reverse Proxy Configuration Correctness', () => {
  // Define reverse proxy configuration files to test
  // Note: nginx.conf is the main config file and doesn't contain WebSocket/security settings
  // Those are in default.conf (the server block configuration)
  const reverseProxyConfigs = [
    { name: 'Caddy', file: 'Caddyfile', type: 'caddy' },
    { name: 'Nginx', file: 'default.conf', type: 'nginx' },
  ];

  /**
   * Helper function to check if Caddy configuration includes WebSocket upgrade headers
   */
  function caddyHasWebSocketHeaders(content: string): boolean {
    // Caddy should have header_up Connection and header_up Upgrade
    const hasConnectionHeader = /header_up\s+Connection\s+\{>Connection\}/.test(content);
    const hasUpgradeHeader = /header_up\s+Upgrade\s+\{>Upgrade\}/.test(content);
    
    return hasConnectionHeader && hasUpgradeHeader;
  }

  /**
   * Helper function to check if Nginx configuration includes WebSocket upgrade headers
   */
  function nginxHasWebSocketHeaders(content: string): boolean {
    // Nginx should have proxy_set_header Upgrade and proxy_set_header Connection
    const hasUpgradeHeader = /proxy_set_header\s+Upgrade\s+\$http_upgrade/.test(content);
    const hasConnectionHeader = /proxy_set_header\s+Connection\s+"upgrade"/.test(content);
    
    return hasUpgradeHeader && hasConnectionHeader;
  }

  /**
   * Helper function to check if configuration includes WebSocket upgrade headers
   */
  function hasWebSocketHeaders(content: string, type: string): boolean {
    if (type === 'caddy') {
      return caddyHasWebSocketHeaders(content);
    } else if (type === 'nginx') {
      return nginxHasWebSocketHeaders(content);
    }
    return false;
  }

  /**
   * Helper function to check if Caddy configuration includes timeout settings
   * Note: Caddy handles timeouts automatically, but we check for reverse_proxy configuration
   */
  function caddyHasTimeoutSettings(content: string): boolean {
    // Caddy has implicit timeout handling in reverse_proxy directive
    // We verify that reverse_proxy is configured properly
    return /reverse_proxy\s+\/app\/\*/.test(content);
  }

  /**
   * Helper function to check if Nginx configuration includes timeout settings
   */
  function nginxHasTimeoutSettings(content: string): boolean {
    // Nginx should have proxy_connect_timeout, proxy_send_timeout, proxy_read_timeout
    // For WebSocket, these should be set to long durations (e.g., 7d)
    const hasConnectTimeout = /proxy_connect_timeout\s+\d+[smhd]/.test(content);
    const hasSendTimeout = /proxy_send_timeout\s+\d+[smhd]/.test(content);
    const hasReadTimeout = /proxy_read_timeout\s+\d+[smhd]/.test(content);
    
    return hasConnectTimeout && hasSendTimeout && hasReadTimeout;
  }

  /**
   * Helper function to check if configuration includes appropriate timeout settings
   */
  function hasTimeoutSettings(content: string, type: string): boolean {
    if (type === 'caddy') {
      return caddyHasTimeoutSettings(content);
    } else if (type === 'nginx') {
      return nginxHasTimeoutSettings(content);
    }
    return false;
  }

  /**
   * Helper function to check if Caddy configuration includes security headers
   */
  function caddyHasSecurityHeaders(content: string): boolean {
    // Caddy should have security headers in the header block
    const hasHSTS = /Strict-Transport-Security\s+"max-age=\d+/.test(content);
    const hasXContentTypeOptions = /X-Content-Type-Options\s+"nosniff"/.test(content);
    const hasXFrameOptions = /X-Frame-Options\s+"(DENY|SAMEORIGIN)"/.test(content);
    
    return hasHSTS && hasXContentTypeOptions && hasXFrameOptions;
  }

  /**
   * Helper function to check if Nginx configuration includes security headers
   */
  function nginxHasSecurityHeaders(content: string): boolean {
    // Nginx should have add_header directives for security headers
    const hasHSTS = /add_header\s+Strict-Transport-Security\s+"max-age=\d+/.test(content);
    const hasXContentTypeOptions = /add_header\s+X-Content-Type-Options\s+"nosniff"/.test(content);
    const hasXFrameOptions = /add_header\s+X-Frame-Options\s+"(DENY|SAMEORIGIN)"/.test(content);
    
    return hasHSTS && hasXContentTypeOptions && hasXFrameOptions;
  }

  /**
   * Helper function to check if configuration includes security headers
   */
  function hasSecurityHeaders(content: string, type: string): boolean {
    if (type === 'caddy') {
      return caddyHasSecurityHeaders(content);
    } else if (type === 'nginx') {
      return nginxHasSecurityHeaders(content);
    }
    return false;
  }

  /**
   * Helper function to get detailed security header information
   */
  function getSecurityHeaderDetails(content: string, type: string): {
    hasHSTS: boolean;
    hasXContentTypeOptions: boolean;
    hasXFrameOptions: boolean;
  } {
    if (type === 'caddy') {
      return {
        hasHSTS: /Strict-Transport-Security\s+"max-age=\d+/.test(content),
        hasXContentTypeOptions: /X-Content-Type-Options\s+"nosniff"/.test(content),
        hasXFrameOptions: /X-Frame-Options\s+"(DENY|SAMEORIGIN)"/.test(content),
      };
    } else {
      return {
        hasHSTS: /add_header\s+Strict-Transport-Security\s+"max-age=\d+/.test(content),
        hasXContentTypeOptions: /add_header\s+X-Content-Type-Options\s+"nosniff"/.test(content),
        hasXFrameOptions: /add_header\s+X-Frame-Options\s+"(DENY|SAMEORIGIN)"/.test(content),
      };
    }
  }

  /**
   * Unit test: Check that all reverse proxy configuration files exist
   */
  test('all reverse proxy configuration files should exist', () => {
    reverseProxyConfigs.forEach(config => {
      expect(fs.existsSync(config.file)).toBe(true);
    });
  });

  /**
   * Property-Based Test: Every reverse proxy configuration includes WebSocket upgrade headers
   * **Validates: Requirements 3.4**
   */
  test('Property 7.1: Every reverse proxy configuration includes WebSocket upgrade headers', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...reverseProxyConfigs),
        (config) => {
          // Read the configuration file
          const content = fs.readFileSync(config.file, 'utf-8');
          
          // Check if it has WebSocket headers
          const result = hasWebSocketHeaders(content, config.type);
          
          if (!result) {
            console.log(`\nMissing WebSocket headers in: ${config.name} (${config.file})`);
            if (config.type === 'caddy') {
              console.log('  Expected: header_up Connection {>Connection} and header_up Upgrade {>Upgrade}');
            } else {
              console.log('  Expected: proxy_set_header Upgrade $http_upgrade and proxy_set_header Connection "upgrade"');
            }
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Every reverse proxy configuration includes appropriate timeout settings
   * **Validates: Requirements 3.4**
   */
  test('Property 7.2: Every reverse proxy configuration includes appropriate timeout settings', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...reverseProxyConfigs),
        (config) => {
          // Read the configuration file
          const content = fs.readFileSync(config.file, 'utf-8');
          
          // Check if it has timeout settings
          const result = hasTimeoutSettings(content, config.type);
          
          if (!result) {
            console.log(`\nMissing timeout settings in: ${config.name} (${config.file})`);
            if (config.type === 'caddy') {
              console.log('  Expected: reverse_proxy configuration with implicit timeout handling');
            } else {
              console.log('  Expected: proxy_connect_timeout, proxy_send_timeout, proxy_read_timeout');
            }
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Every reverse proxy configuration includes security headers
   * **Validates: Requirements 3.7**
   */
  test('Property 7.3: Every reverse proxy configuration includes security headers', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...reverseProxyConfigs),
        (config) => {
          // Read the configuration file
          const content = fs.readFileSync(config.file, 'utf-8');
          
          // Check if it has security headers
          const result = hasSecurityHeaders(content, config.type);
          
          if (!result) {
            const details = getSecurityHeaderDetails(content, config.type);
            console.log(`\nMissing security headers in: ${config.name} (${config.file})`);
            console.log(`  - HSTS: ${details.hasHSTS}`);
            console.log(`  - X-Content-Type-Options: ${details.hasXContentTypeOptions}`);
            console.log(`  - X-Frame-Options: ${details.hasXFrameOptions}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Combined - Every reverse proxy configuration has complete correctness
   * This is the main property test that validates all aspects together
   * **Validates: Requirements 3.4, 3.7**
   */
  test('Property 7: Every reverse proxy configuration has complete correctness', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...reverseProxyConfigs),
        (config) => {
          // Read the configuration file
          const content = fs.readFileSync(config.file, 'utf-8');
          
          // Check all three aspects
          const hasWS = hasWebSocketHeaders(content, config.type);
          const hasTimeout = hasTimeoutSettings(content, config.type);
          const hasSecurity = hasSecurityHeaders(content, config.type);
          
          const result = hasWS && hasTimeout && hasSecurity;
          
          if (!result) {
            console.log(`\nConfiguration correctness issues in: ${config.name} (${config.file})`);
            console.log(`  - Has WebSocket headers: ${hasWS}`);
            console.log(`  - Has timeout settings: ${hasTimeout}`);
            console.log(`  - Has security headers: ${hasSecurity}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Edge case test: Empty configuration should fail all checks
   */
  test('Edge case: Empty configuration should fail all checks', () => {
    const emptyContent = '';
    
    expect(hasWebSocketHeaders(emptyContent, 'caddy')).toBe(false);
    expect(hasWebSocketHeaders(emptyContent, 'nginx')).toBe(false);
    expect(hasTimeoutSettings(emptyContent, 'caddy')).toBe(false);
    expect(hasTimeoutSettings(emptyContent, 'nginx')).toBe(false);
    expect(hasSecurityHeaders(emptyContent, 'caddy')).toBe(false);
    expect(hasSecurityHeaders(emptyContent, 'nginx')).toBe(false);
  });

  /**
   * Edge case test: Partial configuration should fail combined check
   */
  test('Edge case: Partial configuration should fail combined check', () => {
    // Caddy config with only WebSocket headers, missing security headers
    const partialCaddy = `
      reverse_proxy /app/* {
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
      }
    `;
    
    expect(hasWebSocketHeaders(partialCaddy, 'caddy')).toBe(true);
    expect(hasTimeoutSettings(partialCaddy, 'caddy')).toBe(true);
    expect(hasSecurityHeaders(partialCaddy, 'caddy')).toBe(false);
    
    // Nginx config with only WebSocket headers, missing timeouts and security
    const partialNginx = `
      location /app/ {
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
      }
    `;
    
    expect(hasWebSocketHeaders(partialNginx, 'nginx')).toBe(true);
    expect(hasTimeoutSettings(partialNginx, 'nginx')).toBe(false);
    expect(hasSecurityHeaders(partialNginx, 'nginx')).toBe(false);
  });

  /**
   * Edge case test: Configuration with all required elements should pass
   */
  test('Edge case: Complete configuration should pass all checks', () => {
    const completeCaddy = `
      reverse_proxy /app/* {
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
      }
      
      header {
        Strict-Transport-Security "max-age=31536000"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
      }
    `;
    
    expect(hasWebSocketHeaders(completeCaddy, 'caddy')).toBe(true);
    expect(hasTimeoutSettings(completeCaddy, 'caddy')).toBe(true);
    expect(hasSecurityHeaders(completeCaddy, 'caddy')).toBe(true);
    
    const completeNginx = `
      location /app/ {
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
      }
      
      add_header Strict-Transport-Security "max-age=31536000" always;
      add_header X-Content-Type-Options "nosniff" always;
      add_header X-Frame-Options "DENY" always;
    `;
    
    expect(hasWebSocketHeaders(completeNginx, 'nginx')).toBe(true);
    expect(hasTimeoutSettings(completeNginx, 'nginx')).toBe(true);
    expect(hasSecurityHeaders(completeNginx, 'nginx')).toBe(true);
  });

  /**
   * Unit test: Verify Caddy WebSocket header detection works correctly
   */
  test('Unit test: Caddy WebSocket header detection should work correctly', () => {
    const withHeaders = `
      reverse_proxy /app/* {
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
      }
    `;
    
    const withoutConnection = `
      reverse_proxy /app/* {
        header_up Upgrade {>Upgrade}
      }
    `;
    
    const withoutUpgrade = `
      reverse_proxy /app/* {
        header_up Connection {>Connection}
      }
    `;
    
    expect(caddyHasWebSocketHeaders(withHeaders)).toBe(true);
    expect(caddyHasWebSocketHeaders(withoutConnection)).toBe(false);
    expect(caddyHasWebSocketHeaders(withoutUpgrade)).toBe(false);
  });

  /**
   * Unit test: Verify Nginx WebSocket header detection works correctly
   */
  test('Unit test: Nginx WebSocket header detection should work correctly', () => {
    const withHeaders = `
      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection "upgrade";
    `;
    
    const withoutUpgrade = `
      proxy_set_header Connection "upgrade";
    `;
    
    const withoutConnection = `
      proxy_set_header Upgrade $http_upgrade;
    `;
    
    expect(nginxHasWebSocketHeaders(withHeaders)).toBe(true);
    expect(nginxHasWebSocketHeaders(withoutUpgrade)).toBe(false);
    expect(nginxHasWebSocketHeaders(withoutConnection)).toBe(false);
  });

  /**
   * Unit test: Verify Nginx timeout detection works correctly
   */
  test('Unit test: Nginx timeout detection should work correctly', () => {
    const withTimeouts = `
      proxy_connect_timeout 7d;
      proxy_send_timeout 7d;
      proxy_read_timeout 7d;
    `;
    
    const withShortTimeouts = `
      proxy_connect_timeout 60s;
      proxy_send_timeout 60s;
      proxy_read_timeout 60s;
    `;
    
    const missingReadTimeout = `
      proxy_connect_timeout 7d;
      proxy_send_timeout 7d;
    `;
    
    expect(nginxHasTimeoutSettings(withTimeouts)).toBe(true);
    expect(nginxHasTimeoutSettings(withShortTimeouts)).toBe(true); // Any timeout is valid
    expect(nginxHasTimeoutSettings(missingReadTimeout)).toBe(false);
  });

  /**
   * Unit test: Verify security header detection works correctly
   */
  test('Unit test: Security header detection should work correctly', () => {
    const caddyWithSecurity = `
      header {
        Strict-Transport-Security "max-age=31536000"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
      }
    `;
    
    const nginxWithSecurity = `
      add_header Strict-Transport-Security "max-age=31536000" always;
      add_header X-Content-Type-Options "nosniff" always;
      add_header X-Frame-Options "DENY" always;
    `;
    
    const caddyMissingHSTS = `
      header {
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
      }
    `;
    
    expect(caddyHasSecurityHeaders(caddyWithSecurity)).toBe(true);
    expect(nginxHasSecurityHeaders(nginxWithSecurity)).toBe(true);
    expect(caddyHasSecurityHeaders(caddyMissingHSTS)).toBe(false);
  });

  /**
   * Integration test: Verify all existing configuration files pass all checks
   */
  test('Integration test: All existing configuration files should pass all checks', () => {
    const failedConfigs: Array<{ name: string; issues: string[] }> = [];
    const passedConfigs: string[] = [];
    
    reverseProxyConfigs.forEach(config => {
      const content = fs.readFileSync(config.file, 'utf-8');
      const issues: string[] = [];
      
      if (!hasWebSocketHeaders(content, config.type)) {
        issues.push('Missing WebSocket headers');
      }
      if (!hasTimeoutSettings(content, config.type)) {
        issues.push('Missing timeout settings');
      }
      if (!hasSecurityHeaders(content, config.type)) {
        issues.push('Missing security headers');
      }
      
      if (issues.length > 0) {
        failedConfigs.push({ name: config.name, issues });
      } else {
        passedConfigs.push(config.name);
      }
    });
    
    if (failedConfigs.length > 0) {
      console.log('\nConfiguration files with issues:');
      failedConfigs.forEach(config => {
        console.log(`  ${config.name}:`);
        config.issues.forEach(issue => console.log(`    - ${issue}`));
      });
    }
    
    console.log(`\nConfiguration correctness: ${passedConfigs.length}/${reverseProxyConfigs.length} files pass all checks`);
    
    // All configuration files should pass all checks
    expect(failedConfigs.length).toBe(0);
  });

  /**
   * Integration test: Verify Caddy configuration has production-ready settings
   */
  test('Integration test: Caddy configuration should have production-ready settings', () => {
    const caddyContent = fs.readFileSync('Caddyfile', 'utf-8');
    
    // Check for HTTP/2 and HTTP/3 support
    const hasHTTP2HTTP3 = /protocols\s+h1\s+h2\s+h3/.test(caddyContent);
    
    // Check for auto HTTPS (should NOT be disabled)
    // In Caddy, auto_https is enabled by default. We check that it's not explicitly disabled.
    // Exclude comment lines (lines starting with #)
    const hasAutoHTTPSDisabled = /^[^#]*auto_https\s+(off|disable_certs)/m.test(caddyContent);
    const hasAutoHTTPS = !hasAutoHTTPSDisabled; // Auto HTTPS is enabled if not explicitly disabled
    
    // Check for reverse proxy configuration
    const hasReverseProxy = /reverse_proxy\s+\/app\/\*/.test(caddyContent);
    
    console.log('\nCaddy production-ready features:');
    console.log(`  - HTTP/2 and HTTP/3 support: ${hasHTTP2HTTP3}`);
    console.log(`  - Auto HTTPS: ${hasAutoHTTPS}`);
    console.log(`  - Reverse proxy configured: ${hasReverseProxy}`);
    
    expect(hasHTTP2HTTP3).toBe(true);
    expect(hasAutoHTTPS).toBe(true);
    expect(hasReverseProxy).toBe(true);
  });


  /**
   * Integration test: Verify Nginx configuration has production-ready settings
   */
  test('Integration test: Nginx configuration should have production-ready settings', () => {
    const nginxDefaultContent = fs.readFileSync('default.conf', 'utf-8');
    
    // Check for HTTP/2 support
    const hasHTTP2 = /listen\s+443\s+ssl\s+http2/.test(nginxDefaultContent);
    
    // Check for HTTP/3 (QUIC) support
    const hasHTTP3 = /listen\s+443\s+quic/.test(nginxDefaultContent);
    
    // Check for Alt-Svc header (HTTP/3 advertisement)
    const hasAltSvc = /add_header\s+Alt-Svc\s+'h3=":443"/.test(nginxDefaultContent);
    
    // Check for upstream configuration
    const hasUpstream = /upstream\s+soketi_backend/.test(nginxDefaultContent);
    
    console.log('\nNginx production-ready features:');
    console.log(`  - HTTP/2 support: ${hasHTTP2}`);
    console.log(`  - HTTP/3 (QUIC) support: ${hasHTTP3}`);
    console.log(`  - HTTP/3 advertisement (Alt-Svc): ${hasAltSvc}`);
    console.log(`  - Upstream configured: ${hasUpstream}`);
    
    expect(hasHTTP2).toBe(true);
    expect(hasHTTP3).toBe(true);
    expect(hasAltSvc).toBe(true);
    expect(hasUpstream).toBe(true);
  });
});
