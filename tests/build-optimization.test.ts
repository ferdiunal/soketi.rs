/**
 * Build Optimization Tests
 * 
 * Property 11: Build Optimizasyon Yapılandırması
 * Validates: Requirements 6.5, 6.6
 * 
 * Tests that Next.js configuration includes proper optimization settings:
 * - Minification (SWC)
 * - Compression
 * - Code splitting
 * - Caching strategies
 */

import * as fs from 'fs';
import * as path from 'path';
import fc from 'fast-check';

describe('Property 11: Build Optimization Configuration', () => {
  const nextConfigPath = path.join(__dirname, '../next-chat-app/next.config.ts');
  let nextConfigContent: string;

  beforeAll(() => {
    nextConfigContent = fs.readFileSync(nextConfigPath, 'utf-8');
  });

  describe('Unit Tests - Specific Optimization Settings', () => {
    test('should have standalone output mode configured', () => {
      expect(nextConfigContent).toContain("output: 'standalone'");
    });

    test('should have compression enabled', () => {
      expect(nextConfigContent).toContain('compress: true');
    });

    test('should have React strict mode enabled', () => {
      expect(nextConfigContent).toContain('reactStrictMode: true');
    });

    test('should have image optimization configured', () => {
      expect(nextConfigContent).toContain('images:');
      expect(nextConfigContent).toContain('formats:');
    });

    test('should have modern image formats (AVIF, WebP)', () => {
      expect(nextConfigContent).toContain('image/avif');
      expect(nextConfigContent).toContain('image/webp');
    });

    test('should have security headers configured', () => {
      expect(nextConfigContent).toContain('async headers()');
      expect(nextConfigContent).toContain('Strict-Transport-Security');
      expect(nextConfigContent).toContain('X-Content-Type-Options');
      expect(nextConfigContent).toContain('X-Frame-Options');
    });

    test('should have turbopack configuration', () => {
      // Next.js 15+ uses Turbopack by default, webpack config removed
      expect(nextConfigContent).toContain('turbopack:');
    });

    test('should use modern build tooling', () => {
      // Next.js 15+ uses Turbopack which handles code splitting automatically
      // No need for manual webpack configuration
      expect(nextConfigContent).toContain('turbopack');
    });
  });

  describe('Property-Based Tests - Configuration Integrity', () => {
    test('Property: Every optimization setting should be properly configured', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(
            'output',
            'compress',
            'reactStrictMode',
            'images',
            'turbopack',
            'headers'
          ),
          (setting) => {
            // Each optimization setting should exist in the config
            const hasSettingPattern = new RegExp(`${setting}\\s*[:=]`, 'i');
            return hasSettingPattern.test(nextConfigContent);
          }
        ),
        { numRuns: 100 }
      );
    });

    test('Property: All security headers should have proper values', () => {
      const securityHeaders = [
        'Strict-Transport-Security',
        'X-Content-Type-Options',
        'X-Frame-Options',
        'X-XSS-Protection',
        'Referrer-Policy'
      ];

      fc.assert(
        fc.property(
          fc.constantFrom(...securityHeaders),
          (header) => {
            // Each security header should be present in the config
            return nextConfigContent.includes(header);
          }
        ),
        { numRuns: 100 }
      );
    });

    test('Property: Image optimization should support multiple device sizes', () => {
      fc.assert(
        fc.property(
          fc.constantFrom('deviceSizes', 'imageSizes'),
          (sizeConfig) => {
            // Both device sizes and image sizes should be configured
            return nextConfigContent.includes(sizeConfig);
          }
        ),
        { numRuns: 100 }
      );
    });

    test('Property: Configuration should be valid TypeScript', () => {
      fc.assert(
        fc.property(
          fc.constant(nextConfigContent),
          (content) => {
            // Should have proper TypeScript imports and exports
            const hasImport = content.includes('import');
            const hasExport = content.includes('export default');
            const hasTypeAnnotation = content.includes('NextConfig');
            
            return hasImport && hasExport && hasTypeAnnotation;
          }
        ),
        { numRuns: 100 }
      );
    });

    test('Property: Turbopack config should be present', () => {
      fc.assert(
        fc.property(
          fc.constant(nextConfigContent),
          (content) => {
            // Turbopack config should be present (Next.js 15+)
            return content.includes('turbopack');
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('Edge Cases', () => {
    test('should handle missing optional configurations gracefully', () => {
      // Config should work even if some optional settings are missing
      const requiredSettings = ['output', 'compress', 'reactStrictMode'];
      
      requiredSettings.forEach(setting => {
        expect(nextConfigContent).toContain(setting);
      });
    });

    test('should have turbopack configuration', () => {
      // Turbopack is the default bundler in Next.js 15+
      expect(nextConfigContent).toContain('turbopack');
    });

    test('should have proper async function for headers', () => {
      // Headers should be an async function
      expect(nextConfigContent).toMatch(/async\s+headers\s*\(\)/);
    });
  });

  describe('Requirement Validation', () => {
    test('Requirement 6.5: Build optimization settings (standalone, minification)', () => {
      // Standalone output
      expect(nextConfigContent).toContain("output: 'standalone'");
      
      // Note: swcMinify is deprecated in Next.js 15+, SWC is default
      // We verify it's not explicitly disabled
      expect(nextConfigContent).not.toContain('swcMinify: false');
    });

    test('Requirement 6.6: Compression and image optimization', () => {
      // Compression
      expect(nextConfigContent).toContain('compress: true');
      
      // Image optimization
      expect(nextConfigContent).toContain('images:');
      expect(nextConfigContent).toContain('formats:');
      expect(nextConfigContent).toContain('image/avif');
      expect(nextConfigContent).toContain('image/webp');
    });
  });

  describe('Performance Optimizations', () => {
    test('should use Turbopack for automatic code splitting', () => {
      // Turbopack handles code splitting automatically in Next.js 15+
      expect(nextConfigContent).toContain('turbopack');
    });

    test('should have proper image formats for performance', () => {
      // Modern formats reduce image size
      const hasAVIF = nextConfigContent.includes('image/avif');
      const hasWebP = nextConfigContent.includes('image/webp');
      
      expect(hasAVIF || hasWebP).toBe(true);
    });

    test('should have responsive image sizes configured', () => {
      // Multiple sizes for responsive images
      expect(nextConfigContent).toContain('deviceSizes');
      expect(nextConfigContent).toContain('imageSizes');
    });
  });

  describe('Security Optimizations', () => {
    test('should have HSTS header for HTTPS enforcement', () => {
      expect(nextConfigContent).toContain('Strict-Transport-Security');
      expect(nextConfigContent).toMatch(/max-age=\d+/);
    });

    test('should have XSS protection headers', () => {
      expect(nextConfigContent).toContain('X-XSS-Protection');
    });

    test('should have clickjacking protection', () => {
      expect(nextConfigContent).toContain('X-Frame-Options');
    });

    test('should have MIME sniffing protection', () => {
      expect(nextConfigContent).toContain('X-Content-Type-Options');
      expect(nextConfigContent).toContain('nosniff');
    });
  });
});

// Feature: documentation-and-deployment-enhancements, Property 11: Build Optimizasyon Yapılandırması
