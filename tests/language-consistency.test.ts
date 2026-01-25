/**
 * Property-Based Tests for Language Consistency
 * Feature: documentation-and-deployment-enhancements
 * Property 1: Dil Tutarlılığı
 * 
 * **Validates: Requirements 1.2, 2.4**
 * 
 * Tests that for every English documentation file, there must be a corresponding
 * Turkish documentation file and both files must have the same section structure.
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 1: Language Consistency', () => {
  /**
   * Helper function to get all markdown files in a directory recursively
   */
  function getMarkdownFiles(dir: string, baseDir: string = dir): string[] {
    const files: string[] = [];
    
    if (!fs.existsSync(dir)) {
      return files;
    }
    
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        // Recursively get files from subdirectories
        files.push(...getMarkdownFiles(fullPath, baseDir));
      } else if (entry.isFile() && entry.name.endsWith('.md') && entry.name !== '.gitkeep') {
        // Get relative path from base directory
        const relativePath = path.relative(baseDir, fullPath);
        files.push(relativePath);
      }
    }
    
    return files;
  }

  /**
   * Helper function to map English file paths to Turkish file paths
   * Examples:
   * - docs/en/getting-started.md -> docs/tr/baslangic.md
   * - docs/en/api-reference.md -> docs/tr/api-referans.md
   * - docs/en/examples/basic-chat.md -> docs/tr/ornekler/temel-chat.md
   */
  function mapEnglishToTurkish(englishPath: string): string {
    // Define the mapping of English file/folder names to Turkish equivalents
    const nameMapping: Record<string, string> = {
      // Main documentation files
      'getting-started.md': 'baslangic.md',
      'installation.md': 'kurulum.md',
      'configuration.md': 'yapilandirma.md',
      'api-reference.md': 'api-referans.md',
      'troubleshooting.md': 'sorun-giderme.md',
      
      // Folders
      'examples': 'ornekler',
      'deployment': 'deployment', // Same in both languages
      
      // Example files
      'basic-chat.md': 'temel-chat.md',
      'presence.md': 'presence.md', // Same in both languages
      'private-channels.md': 'ozel-kanallar.md',
      'authentication.md': 'kimlik-dogrulama.md',
      
      // Deployment files (same in both languages)
      'vercel.md': 'vercel.md',
      'netlify.md': 'netlify.md',
      'reverse-proxy.md': 'reverse-proxy.md',
    };
    
    // Replace 'en' with 'tr' in the path
    let turkishPath = englishPath.replace(/^en[\/\\]/, 'tr/');
    
    // Replace each path component with its Turkish equivalent if mapping exists
    const pathParts = turkishPath.split(/[\/\\]/);
    const mappedParts = pathParts.map(part => nameMapping[part] || part);
    turkishPath = mappedParts.join('/');
    
    return turkishPath;
  }

  /**
   * Helper function to extract headings from markdown content
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
        const level = headingMatch[1].length;
        const text = headingMatch[2].trim();
        headings.push(`${'#'.repeat(level)} ${text}`);
      }
    }
    
    return headings;
  }

  /**
   * Helper function to normalize heading text for comparison
   * Removes language-specific text but keeps the structure
   */
  function normalizeHeadingStructure(headings: string[]): string[] {
    return headings.map(heading => {
      // Extract just the heading level (number of #)
      const match = heading.match(/^(#{1,6})\s+/);
      if (match) {
        return match[1]; // Return just the # symbols
      }
      return heading;
    });
  }

  /**
   * Helper function to check if two files have the same heading structure
   * Returns true if both files have the same number of headings at each level
   */
  function haveSameHeadingStructure(englishContent: string, turkishContent: string): boolean {
    const englishHeadings = extractHeadings(englishContent);
    const turkishHeadings = extractHeadings(turkishContent);
    
    // Normalize to just heading levels
    const englishStructure = normalizeHeadingStructure(englishHeadings);
    const turkishStructure = normalizeHeadingStructure(turkishHeadings);
    
    // Check if they have the same structure
    if (englishStructure.length !== turkishStructure.length) {
      return false;
    }
    
    // Check each heading level matches
    for (let i = 0; i < englishStructure.length; i++) {
      if (englishStructure[i] !== turkishStructure[i]) {
        return false;
      }
    }
    
    return true;
  }

  /**
   * Helper function to get heading statistics for debugging
   */
  function getHeadingStats(content: string): {
    total: number;
    byLevel: Record<number, number>;
    headings: string[];
  } {
    const headings = extractHeadings(content);
    const byLevel: Record<number, number> = {};
    
    headings.forEach(heading => {
      const level = heading.match(/^(#{1,6})/)?.[1].length || 0;
      byLevel[level] = (byLevel[level] || 0) + 1;
    });
    
    return {
      total: headings.length,
      byLevel,
      headings,
    };
  }

  // Get all English documentation files
  const englishDocsDir = 'docs/en';
  const turkishDocsDir = 'docs/tr';
  const englishFiles = getMarkdownFiles(englishDocsDir, englishDocsDir);

  /**
   * Unit test: Check that English documentation directory exists
   */
  test('English documentation directory should exist', () => {
    expect(fs.existsSync(englishDocsDir)).toBe(true);
  });

  /**
   * Unit test: Check that Turkish documentation directory exists
   */
  test('Turkish documentation directory should exist', () => {
    expect(fs.existsSync(turkishDocsDir)).toBe(true);
  });

  /**
   * Unit test: Check that there are English documentation files
   */
  test('English documentation should have markdown files', () => {
    expect(englishFiles.length).toBeGreaterThan(0);
  });

  /**
   * Property-Based Test: For every English documentation file, a corresponding Turkish file exists
   * **Validates: Requirements 1.2, 2.4**
   */
  test('Property 1.1: For every English documentation file, a corresponding Turkish file exists', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...englishFiles),
        (englishFile) => {
          // Map English file path to Turkish file path
          const turkishFile = mapEnglishToTurkish(englishFile);
          const turkishFullPath = path.join(turkishDocsDir, turkishFile);
          
          const result = fs.existsSync(turkishFullPath);
          
          if (!result) {
            console.log(`\nMissing Turkish translation for: ${englishFile}`);
            console.log(`  Expected Turkish file: ${turkishFile}`);
            console.log(`  Full path: ${turkishFullPath}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: For every English documentation file, the Turkish version has the same heading structure
   * This is the main property test that validates the language consistency requirement
   * **Validates: Requirements 1.2, 2.4**
   */
  test('Property 1: For every English documentation file, the Turkish version has the same heading structure', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...englishFiles),
        (englishFile) => {
          // Get full paths
          const englishFullPath = path.join(englishDocsDir, englishFile);
          const turkishFile = mapEnglishToTurkish(englishFile);
          const turkishFullPath = path.join(turkishDocsDir, turkishFile);
          
          // Check if Turkish file exists
          if (!fs.existsSync(turkishFullPath)) {
            console.log(`\nSkipping structure check - Turkish file does not exist: ${turkishFile}`);
            return false;
          }
          
          // Read both files
          const englishContent = fs.readFileSync(englishFullPath, 'utf-8');
          const turkishContent = fs.readFileSync(turkishFullPath, 'utf-8');
          
          // Check if they have the same heading structure
          const result = haveSameHeadingStructure(englishContent, turkishContent);
          
          if (!result) {
            const englishStats = getHeadingStats(englishContent);
            const turkishStats = getHeadingStats(turkishContent);
            
            console.log(`\nHeading structure mismatch:`);
            console.log(`  English file: ${englishFile}`);
            console.log(`  Turkish file: ${turkishFile}`);
            console.log(`\n  English headings (${englishStats.total} total):`);
            console.log(`    By level:`, englishStats.byLevel);
            englishStats.headings.forEach((h, i) => {
              console.log(`    ${i + 1}. ${h}`);
            });
            console.log(`\n  Turkish headings (${turkishStats.total} total):`);
            console.log(`    By level:`, turkishStats.byLevel);
            turkishStats.headings.forEach((h, i) => {
              console.log(`    ${i + 1}. ${h}`);
            });
          }
          
          return result;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Property-Based Test: Both English and Turkish files have at least one heading
   * **Validates: Requirements 1.2**
   */
  test('Property 1.2: Both English and Turkish documentation files have at least one heading', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...englishFiles),
        (englishFile) => {
          // Get full paths
          const englishFullPath = path.join(englishDocsDir, englishFile);
          const turkishFile = mapEnglishToTurkish(englishFile);
          const turkishFullPath = path.join(turkishDocsDir, turkishFile);
          
          // Check if Turkish file exists
          if (!fs.existsSync(turkishFullPath)) {
            return false;
          }
          
          // Read both files
          const englishContent = fs.readFileSync(englishFullPath, 'utf-8');
          const turkishContent = fs.readFileSync(turkishFullPath, 'utf-8');
          
          // Extract headings
          const englishHeadings = extractHeadings(englishContent);
          const turkishHeadings = extractHeadings(turkishContent);
          
          const result = englishHeadings.length > 0 && turkishHeadings.length > 0;
          
          if (!result) {
            console.log(`\nMissing headings:`);
            console.log(`  English file: ${englishFile} (${englishHeadings.length} headings)`);
            console.log(`  Turkish file: ${turkishFile} (${turkishHeadings.length} headings)`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Edge case test: Empty files should fail the property
   */
  test('Edge case: Empty files should not have matching structure', () => {
    const emptyContent = '';
    const contentWithHeadings = '# Title\n\n## Section\n\nContent';
    
    expect(haveSameHeadingStructure(emptyContent, emptyContent)).toBe(true); // Both empty = same structure
    expect(haveSameHeadingStructure(emptyContent, contentWithHeadings)).toBe(false);
    expect(haveSameHeadingStructure(contentWithHeadings, emptyContent)).toBe(false);
  });

  /**
   * Edge case test: Files with different heading levels should fail
   */
  test('Edge case: Files with different heading levels should not match', () => {
    const content1 = '# Title\n\n## Section 1\n\n## Section 2';
    const content2 = '# Title\n\n## Section 1\n\n### Subsection'; // Different level
    const content3 = '# Title\n\n## Section 1'; // Missing section
    
    expect(haveSameHeadingStructure(content1, content1)).toBe(true);
    expect(haveSameHeadingStructure(content1, content2)).toBe(false);
    expect(haveSameHeadingStructure(content1, content3)).toBe(false);
  });

  /**
   * Edge case test: Headings inside code blocks should be ignored
   */
  test('Edge case: Headings inside code blocks should be ignored', () => {
    const content1 = '# Title\n\n```markdown\n# Not a heading\n```\n\n## Section';
    const content2 = '# Title\n\n## Section';
    
    expect(haveSameHeadingStructure(content1, content2)).toBe(true);
  });

  /**
   * Edge case test: Same structure with different text should match
   */
  test('Edge case: Same structure with different text should match', () => {
    const englishContent = '# Getting Started\n\n## Installation\n\n## Configuration';
    const turkishContent = '# Başlangıç\n\n## Kurulum\n\n## Yapılandırma';
    
    expect(haveSameHeadingStructure(englishContent, turkishContent)).toBe(true);
  });

  /**
   * Unit test: Verify heading extraction works correctly
   */
  test('Unit test: Heading extraction should work correctly', () => {
    const content = `
# Main Title

Some text.

## Section 1

More text.

### Subsection 1.1

\`\`\`markdown
# This is not a heading
## Neither is this
\`\`\`

## Section 2

End.
`;
    
    const headings = extractHeadings(content);
    
    expect(headings.length).toBe(4);
    expect(headings[0]).toMatch(/^# Main Title/);
    expect(headings[1]).toMatch(/^## Section 1/);
    expect(headings[2]).toMatch(/^### Subsection 1.1/);
    expect(headings[3]).toMatch(/^## Section 2/);
  });

  /**
   * Unit test: Verify English to Turkish path mapping works correctly
   */
  test('Unit test: English to Turkish path mapping should work correctly', () => {
    expect(mapEnglishToTurkish('getting-started.md')).toBe('baslangic.md');
    expect(mapEnglishToTurkish('api-reference.md')).toBe('api-referans.md');
    expect(mapEnglishToTurkish('examples/basic-chat.md')).toBe('ornekler/temel-chat.md');
    expect(mapEnglishToTurkish('examples/presence.md')).toBe('ornekler/presence.md');
    expect(mapEnglishToTurkish('deployment/vercel.md')).toBe('deployment/vercel.md');
  });

  /**
   * Integration test: Verify that all existing English files have Turkish counterparts
   */
  test('Integration test: All existing English files should have Turkish counterparts', () => {
    const missingFiles: string[] = [];
    const existingPairs: Array<{ english: string; turkish: string }> = [];
    
    englishFiles.forEach(englishFile => {
      const turkishFile = mapEnglishToTurkish(englishFile);
      const turkishFullPath = path.join(turkishDocsDir, turkishFile);
      
      if (!fs.existsSync(turkishFullPath)) {
        missingFiles.push(turkishFile);
      } else {
        existingPairs.push({ english: englishFile, turkish: turkishFile });
      }
    });
    
    if (missingFiles.length > 0) {
      console.log('\nMissing Turkish translations:');
      missingFiles.forEach(file => console.log(`  - ${file}`));
    }
    
    console.log(`\nLanguage coverage: ${existingPairs.length}/${englishFiles.length} files have Turkish translations`);
    
    // All English files should have Turkish counterparts
    expect(missingFiles.length).toBe(0);
  });

  /**
   * Integration test: Verify that all existing file pairs have matching heading structures
   */
  test('Integration test: All existing file pairs should have matching heading structures', () => {
    const mismatchedFiles: Array<{ english: string; turkish: string }> = [];
    const matchedFiles: Array<{ english: string; turkish: string }> = [];
    
    englishFiles.forEach(englishFile => {
      const englishFullPath = path.join(englishDocsDir, englishFile);
      const turkishFile = mapEnglishToTurkish(englishFile);
      const turkishFullPath = path.join(turkishDocsDir, turkishFile);
      
      if (fs.existsSync(turkishFullPath)) {
        const englishContent = fs.readFileSync(englishFullPath, 'utf-8');
        const turkishContent = fs.readFileSync(turkishFullPath, 'utf-8');
        
        if (haveSameHeadingStructure(englishContent, turkishContent)) {
          matchedFiles.push({ english: englishFile, turkish: turkishFile });
        } else {
          mismatchedFiles.push({ english: englishFile, turkish: turkishFile });
        }
      }
    });
    
    if (mismatchedFiles.length > 0) {
      console.log('\nFiles with mismatched heading structures:');
      mismatchedFiles.forEach(pair => {
        console.log(`  - ${pair.english} <-> ${pair.turkish}`);
      });
    }
    
    console.log(`\nStructure consistency: ${matchedFiles.length}/${matchedFiles.length + mismatchedFiles.length} file pairs have matching structures`);
    
    // All file pairs should have matching structures
    expect(mismatchedFiles.length).toBe(0);
  });
});
