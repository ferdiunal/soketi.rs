/**
 * Property-Based Tests for README Files
 * Feature: documentation-and-deployment-enhancements
 * Property 5: README Bölüm Varlığı
 * Property 6: README Dokümantasyon Bağlantıları
 * 
 * **Validates: Requirements 2.3, 2.6**
 * 
 * Tests that:
 * - Each README file (README.md and README.tr.md) contains required sections
 * - Each README file contains valid links to the docs/ folder
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 5: README Section Presence', () => {
  // Define the README files to test
  const readmeFiles = ['README.md', 'README.tr.md'];

  // Define required sections for README files
  // These are the sections mentioned in Requirement 2.3
  const requiredSections = [
    'project overview',    // Project overview section
    'quick start',         // Quick start section
    'installation',        // Installation section
    'configuration',       // Configuration section
    'usage examples',      // Usage examples section
  ];

  /**
   * Helper function to extract all headings from markdown content
   * Ignores headings inside code blocks
   */
  function extractHeadings(content: string): string[] {
    const lines = content.split('\n');
    const headings: string[] = [];
    let inCodeBlock = false;

    for (const line of lines) {
      // Toggle code block state
      if (line.trim().startsWith('```')) {
        inCodeBlock = !inCodeBlock;
        continue;
      }

      // Skip lines inside code blocks
      if (inCodeBlock) {
        continue;
      }

      // Check if line is a heading
      const headingMatch = line.match(/^(#{1,6})\s+(.+)$/);
      if (headingMatch) {
        const text = headingMatch[2].trim();
        headings.push(text);
      }
    }

    return headings;
  }

  /**
   * Helper function to check if a section exists in the content
   * Performs case-insensitive matching and handles variations
   */
  function hasSectionWithKeywords(headings: string[], keywords: string[]): boolean {
    const normalizedHeadings = headings.map(h => h.toLowerCase());
    
    // Check if any heading contains all keywords (in any order)
    return normalizedHeadings.some(heading => {
      return keywords.every(keyword => heading.includes(keyword.toLowerCase()));
    });
  }

  /**
   * Helper function to check if README has a section matching the concept
   * Handles both English and Turkish variations
   */
  function hasRequiredSection(content: string, sectionConcept: string): boolean {
    const headings = extractHeadings(content);

    // Define keyword mappings for each required section
    // Supports both English and Turkish variations
    const sectionKeywords: Record<string, string[][]> = {
      'project overview': [
        ['features'],           // Features section
        ['overview'],           // Overview section
        ['about'],              // About section
        ['özellikler'],         // Turkish: Features
        ['hakkında'],           // Turkish: About
        ['genel', 'bakış'],     // Turkish: Overview
      ],
      'quick start': [
        ['quick', 'start'],     // Quick Start
        ['hızlı', 'başlangıç'], // Turkish: Quick Start
        ['getting', 'started'], // Getting Started
        ['başlangıç'],          // Turkish: Start
      ],
      'installation': [
        ['installation'],       // Installation
        ['install'],            // Install
        ['kurulum'],            // Turkish: Installation
      ],
      'configuration': [
        ['configuration'],      // Configuration
        ['config'],             // Config
        ['yapılandırma'],       // Turkish: Configuration
      ],
      'usage examples': [
        ['usage', 'examples'],  // Usage Examples
        ['examples'],           // Examples
        ['kullanım', 'örnekleri'], // Turkish: Usage Examples
        ['örnekler'],           // Turkish: Examples
        ['ornekler'],           // Turkish: Examples (without special char)
      ],
    };

    const keywordSets = sectionKeywords[sectionConcept] || [];

    // Check if any keyword set matches
    return keywordSets.some(keywords => hasSectionWithKeywords(headings, keywords));
  }

  /**
   * Unit test: Check that README.md exists
   */
  test('README.md should exist', () => {
    expect(fs.existsSync('README.md')).toBe(true);
  });

  /**
   * Unit test: Check that README.tr.md exists
   */
  test('README.tr.md should exist', () => {
    expect(fs.existsSync('README.tr.md')).toBe(true);
  });

  /**
   * Unit test: Check that README files are not empty
   */
  test('README files should not be empty', () => {
    readmeFiles.forEach(file => {
      if (fs.existsSync(file)) {
        const content = fs.readFileSync(file, 'utf-8');
        expect(content.length).toBeGreaterThan(0);
      }
    });
  });

  /**
   * Property-Based Test: Each README file contains all required sections
   * **Validates: Requirements 2.3**
   */
  test('Property 5: Each README file contains all required sections', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...readmeFiles),
        fc.constantFrom(...requiredSections),
        (readmeFile, requiredSection) => {
          // Check if file exists
          if (!fs.existsSync(readmeFile)) {
            console.log(`\nREADME file does not exist: ${readmeFile}`);
            return false;
          }

          // Read file content
          const content = fs.readFileSync(readmeFile, 'utf-8');

          // Check if required section exists
          const result = hasRequiredSection(content, requiredSection);

          if (!result) {
            console.log(`\nMissing required section in ${readmeFile}:`);
            console.log(`  Required section: ${requiredSection}`);
            console.log(`\n  Available headings:`);
            const headings = extractHeadings(content);
            headings.slice(0, 20).forEach((h, i) => {
              console.log(`    ${i + 1}. ${h}`);
            });
            if (headings.length > 20) {
              console.log(`    ... and ${headings.length - 20} more`);
            }
          }

          return result;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Integration test: Verify all required sections exist in each README
   */
  test('Integration test: All required sections should exist in each README', () => {
    const results: Record<string, Record<string, boolean>> = {};

    readmeFiles.forEach(readmeFile => {
      if (!fs.existsSync(readmeFile)) {
        console.log(`\nSkipping ${readmeFile} - file does not exist`);
        return;
      }

      const content = fs.readFileSync(readmeFile, 'utf-8');
      results[readmeFile] = {};

      requiredSections.forEach(section => {
        results[readmeFile][section] = hasRequiredSection(content, section);
      });
    });

    // Print results
    console.log('\nREADME Section Coverage:');
    Object.entries(results).forEach(([file, sections]) => {
      const present = Object.values(sections).filter(v => v).length;
      const total = Object.values(sections).length;
      console.log(`\n  ${file}: ${present}/${total} sections present`);
      
      Object.entries(sections).forEach(([section, hasIt]) => {
        const status = hasIt ? '✓' : '✗';
        console.log(`    ${status} ${section}`);
      });
    });

    // All sections should be present in all README files
    Object.entries(results).forEach(([file, sections]) => {
      Object.entries(sections).forEach(([section, hasIt]) => {
        expect(hasIt).toBe(true);
      });
    });
  });

  /**
   * Edge case test: Empty content should not have any sections
   */
  test('Edge case: Empty content should not have required sections', () => {
    const emptyContent = '';
    requiredSections.forEach(section => {
      expect(hasRequiredSection(emptyContent, section)).toBe(false);
    });
  });

  /**
   * Edge case test: Content with only code blocks should not match
   */
  test('Edge case: Headings in code blocks should not count as sections', () => {
    const contentWithCodeBlock = '```markdown\n# Installation\n## Configuration\n```';
    expect(hasRequiredSection(contentWithCodeBlock, 'installation')).toBe(false);
    expect(hasRequiredSection(contentWithCodeBlock, 'configuration')).toBe(false);
  });

  /**
   * Edge case test: Section matching should be case-insensitive
   */
  test('Edge case: Section matching should be case-insensitive', () => {
    const content1 = '# INSTALLATION\n\nSome content.';
    const content2 = '# installation\n\nSome content.';
    const content3 = '# Installation\n\nSome content.';

    expect(hasRequiredSection(content1, 'installation')).toBe(true);
    expect(hasRequiredSection(content2, 'installation')).toBe(true);
    expect(hasRequiredSection(content3, 'installation')).toBe(true);
  });

  /**
   * Edge case test: Partial keyword matches should work
   */
  test('Edge case: Partial keyword matches should work', () => {
    const content = '# Quick Start Guide\n\nContent here.';
    expect(hasRequiredSection(content, 'quick start')).toBe(true);
  });

  /**
   * Unit test: Verify heading extraction works correctly
   */
  test('Unit test: Heading extraction should work correctly', () => {
    const content = `
# Main Title

Some text.

## Installation

More text.

### Configuration Details

\`\`\`markdown
# This is not a heading
\`\`\`

## Usage Examples

End.
`;

    const headings = extractHeadings(content);

    expect(headings.length).toBe(4);
    expect(headings).toContain('Main Title');
    expect(headings).toContain('Installation');
    expect(headings).toContain('Configuration Details');
    expect(headings).toContain('Usage Examples');
    expect(headings).not.toContain('This is not a heading');
  });
});

describe('Property 6: README Documentation Links', () => {
  const readmeFiles = ['README.md', 'README.tr.md'];

  /**
   * Helper function to extract all markdown links from content
   * Returns array of { text, url } objects
   */
  function extractMarkdownLinks(content: string): Array<{ text: string; url: string }> {
    const links: Array<{ text: string; url: string }> = [];
    const lines = content.split('\n');
    let inCodeBlock = false;

    for (const line of lines) {
      // Toggle code block state
      if (line.trim().startsWith('```')) {
        inCodeBlock = !inCodeBlock;
        continue;
      }

      // Skip lines inside code blocks
      if (inCodeBlock) {
        continue;
      }

      // Match markdown links: [text](url)
      const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
      let match;

      while ((match = linkRegex.exec(line)) !== null) {
        links.push({
          text: match[1],
          url: match[2],
        });
      }
    }

    return links;
  }

  /**
   * Helper function to check if a link points to the docs/ folder
   */
  function isDocsLink(url: string): boolean {
    // Remove any anchor fragments
    const urlWithoutAnchor = url.split('#')[0];
    
    // Check if it's a relative path to docs/
    return urlWithoutAnchor.startsWith('docs/') || 
           urlWithoutAnchor.startsWith('./docs/') ||
           urlWithoutAnchor.startsWith('../docs/');
  }

  /**
   * Helper function to resolve relative path and check if file exists
   */
  function linkTargetExists(url: string, baseDir: string = '.'): boolean {
    // Remove anchor fragments
    const urlWithoutAnchor = url.split('#')[0];
    
    // Skip external URLs
    if (urlWithoutAnchor.startsWith('http://') || urlWithoutAnchor.startsWith('https://')) {
      return true; // We don't validate external URLs
    }

    // Resolve relative path
    const fullPath = path.join(baseDir, urlWithoutAnchor);
    
    return fs.existsSync(fullPath);
  }

  /**
   * Unit test: Check that README files contain links
   */
  test('README files should contain markdown links', () => {
    readmeFiles.forEach(file => {
      if (fs.existsSync(file)) {
        const content = fs.readFileSync(file, 'utf-8');
        const links = extractMarkdownLinks(content);
        expect(links.length).toBeGreaterThan(0);
      }
    });
  });

  /**
   * Property-Based Test: Each README file contains valid links to docs/ folder
   * **Validates: Requirements 2.6**
   */
  test('Property 6: Each README file contains valid links to docs/ folder', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...readmeFiles),
        (readmeFile) => {
          // Check if file exists
          if (!fs.existsSync(readmeFile)) {
            console.log(`\nREADME file does not exist: ${readmeFile}`);
            return false;
          }

          // Read file content
          const content = fs.readFileSync(readmeFile, 'utf-8');

          // Extract all links
          const allLinks = extractMarkdownLinks(content);

          // Filter to only docs/ links
          const docsLinks = allLinks.filter(link => isDocsLink(link.url));

          // Check that there is at least one docs/ link
          if (docsLinks.length === 0) {
            console.log(`\nNo docs/ links found in ${readmeFile}`);
            console.log(`  Total links found: ${allLinks.length}`);
            return false;
          }

          // Check that all docs/ links point to existing files
          const invalidLinks = docsLinks.filter(link => !linkTargetExists(link.url));

          if (invalidLinks.length > 0) {
            console.log(`\nInvalid docs/ links in ${readmeFile}:`);
            invalidLinks.forEach(link => {
              console.log(`  - [${link.text}](${link.url})`);
            });
            return false;
          }

          return true;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Property-Based Test: All docs/ links in README files point to existing files
   * **Validates: Requirements 2.6**
   */
  test('Property 6.1: All docs/ links in README files point to existing files', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...readmeFiles),
        (readmeFile) => {
          // Check if file exists
          if (!fs.existsSync(readmeFile)) {
            return false;
          }

          // Read file content
          const content = fs.readFileSync(readmeFile, 'utf-8');

          // Extract all links
          const allLinks = extractMarkdownLinks(content);

          // Filter to only docs/ links
          const docsLinks = allLinks.filter(link => isDocsLink(link.url));

          // Check each docs/ link
          const results = docsLinks.map(link => ({
            link,
            exists: linkTargetExists(link.url),
          }));

          const invalidLinks = results.filter(r => !r.exists);

          if (invalidLinks.length > 0) {
            console.log(`\nBroken docs/ links in ${readmeFile}:`);
            invalidLinks.forEach(({ link }) => {
              console.log(`  - [${link.text}](${link.url})`);
            });
          }

          return invalidLinks.length === 0;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Integration test: Verify all README files have docs/ links
   */
  test('Integration test: All README files should have docs/ links', () => {
    const results: Record<string, { total: number; docsLinks: number; valid: number; invalid: Array<{ text: string; url: string }> }> = {};

    readmeFiles.forEach(readmeFile => {
      if (!fs.existsSync(readmeFile)) {
        console.log(`\nSkipping ${readmeFile} - file does not exist`);
        return;
      }

      const content = fs.readFileSync(readmeFile, 'utf-8');
      const allLinks = extractMarkdownLinks(content);
      const docsLinks = allLinks.filter(link => isDocsLink(link.url));
      const validLinks = docsLinks.filter(link => linkTargetExists(link.url));
      const invalidLinks = docsLinks.filter(link => !linkTargetExists(link.url));

      results[readmeFile] = {
        total: allLinks.length,
        docsLinks: docsLinks.length,
        valid: validLinks.length,
        invalid: invalidLinks.map(link => ({ text: link.text, url: link.url })),
      };
    });

    // Print results
    console.log('\nREADME Documentation Links:');
    Object.entries(results).forEach(([file, stats]) => {
      console.log(`\n  ${file}:`);
      console.log(`    Total links: ${stats.total}`);
      console.log(`    Links to docs/: ${stats.docsLinks}`);
      console.log(`    Valid docs/ links: ${stats.valid}`);
      console.log(`    Invalid docs/ links: ${stats.invalid.length}`);

      if (stats.invalid.length > 0) {
        console.log(`\n    Broken links:`);
        stats.invalid.forEach(link => {
          console.log(`      - [${link.text}](${link.url})`);
        });
      }
    });

    // All README files should have at least one docs/ link
    Object.entries(results).forEach(([file, stats]) => {
      expect(stats.docsLinks).toBeGreaterThan(0);
    });

    // All docs/ links should be valid
    Object.entries(results).forEach(([file, stats]) => {
      expect(stats.invalid.length).toBe(0);
    });
  });

  /**
   * Edge case test: Links in code blocks should not be extracted
   */
  test('Edge case: Links in code blocks should not be extracted', () => {
    const content = '```markdown\n[Link](docs/test.md)\n```';
    const links = extractMarkdownLinks(content);
    expect(links.length).toBe(0);
  });

  /**
   * Edge case test: External links should not be considered docs/ links
   */
  test('Edge case: External links should not be considered docs/ links', () => {
    expect(isDocsLink('https://example.com/docs/test.md')).toBe(false);
    expect(isDocsLink('http://example.com/docs/test.md')).toBe(false);
    expect(isDocsLink('docs/test.md')).toBe(true);
    expect(isDocsLink('./docs/test.md')).toBe(true);
  });

  /**
   * Edge case test: Links with anchors should work
   */
  test('Edge case: Links with anchors should work', () => {
    const content = '[Link](docs/en/getting-started.md#installation)';
    const links = extractMarkdownLinks(content);
    
    expect(links.length).toBe(1);
    expect(isDocsLink(links[0].url)).toBe(true);
    
    // Should check file existence without anchor
    expect(linkTargetExists('docs/en/getting-started.md#installation')).toBe(
      fs.existsSync('docs/en/getting-started.md')
    );
  });

  /**
   * Unit test: Verify link extraction works correctly
   */
  test('Unit test: Link extraction should work correctly', () => {
    const content = `
# Title

Check out the [Getting Started](docs/en/getting-started.md) guide.

Also see:
- [API Reference](docs/en/api-reference.md)
- [Configuration](docs/en/configuration.md#basic)
- [External Link](https://example.com)

\`\`\`markdown
[Not Extracted](docs/test.md)
\`\`\`
`;

    const links = extractMarkdownLinks(content);

    expect(links.length).toBe(4);
    expect(links[0]).toEqual({ text: 'Getting Started', url: 'docs/en/getting-started.md' });
    expect(links[1]).toEqual({ text: 'API Reference', url: 'docs/en/api-reference.md' });
    expect(links[2]).toEqual({ text: 'Configuration', url: 'docs/en/configuration.md#basic' });
    expect(links[3]).toEqual({ text: 'External Link', url: 'https://example.com' });
  });

  /**
   * Unit test: Verify docs/ link filtering works correctly
   */
  test('Unit test: Docs link filtering should work correctly', () => {
    const links = [
      { text: 'Docs Link 1', url: 'docs/en/test.md' },
      { text: 'Docs Link 2', url: './docs/tr/test.md' },
      { text: 'External', url: 'https://example.com' },
      { text: 'Relative', url: '../README.md' },
      { text: 'Docs Link 3', url: 'docs/en/examples/test.md' },
    ];

    const docsLinks = links.filter(link => isDocsLink(link.url));

    expect(docsLinks.length).toBe(3);
    expect(docsLinks[0].text).toBe('Docs Link 1');
    expect(docsLinks[1].text).toBe('Docs Link 2');
    expect(docsLinks[2].text).toBe('Docs Link 3');
  });
});
