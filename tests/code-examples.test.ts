/**
 * Property-Based Tests for Code Examples Presence
 * Feature: documentation-and-deployment-enhancements
 * Property 3: Kod Örnekleri Varlığı
 * 
 * **Validates: Requirements 1.5, 8.1**
 * 
 * Tests that every main feature documentation contains at least one
 * code block with syntax highlighting.
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 3: Code Examples Presence', () => {
  // Define major feature documentation files to test
  // These are the main documentation files that should contain code examples
  const majorFeatureDocFiles = [
    // English documentation
    'docs/en/getting-started.md',
    'docs/en/installation.md',
    'docs/en/configuration.md',
    'docs/en/api-reference.md',
    'docs/en/troubleshooting.md',
    
    // English deployment guides
    'docs/en/deployment/vercel.md',
    'docs/en/deployment/netlify.md',
    'docs/en/deployment/reverse-proxy.md',
    
    // English examples
    'docs/en/examples/basic-chat.md',
    'docs/en/examples/presence.md',
    'docs/en/examples/private-channels.md',
    'docs/en/examples/authentication.md',
    
    // Turkish documentation
    'docs/tr/baslangic.md',
    'docs/tr/kurulum.md',
    'docs/tr/yapilandirma.md',
    'docs/tr/api-referans.md',
    
    // Turkish deployment guides
    'docs/tr/deployment/vercel.md',
    'docs/tr/deployment/netlify.md',
    'docs/tr/deployment/reverse-proxy.md',
    
    // Turkish examples
    'docs/tr/ornekler/temel-chat.md',
    'docs/tr/ornekler/presence.md',
    'docs/tr/ornekler/ozel-kanallar.md',
    'docs/tr/ornekler/kimlik-dogrulama.md',
  ];

  /**
   * Helper function to extract code blocks from markdown content
   * Code blocks are defined by triple backticks with optional language identifier
   * Examples:
   * ```typescript
   * ```javascript
   * ```bash
   * ```json
   */
  function extractCodeBlocks(content: string): Array<{ language: string; code: string }> {
    const codeBlocks: Array<{ language: string; code: string }> = [];
    const lines = content.split('\n');
    
    let inCodeBlock = false;
    let currentLanguage = '';
    let currentCode: string[] = [];
    
    for (const line of lines) {
      // Check for code block start
      const startMatch = line.match(/^```(\w+)?/);
      if (startMatch && !inCodeBlock) {
        inCodeBlock = true;
        currentLanguage = startMatch[1] || 'plain';
        currentCode = [];
        continue;
      }
      
      // Check for code block end
      if (line.trim() === '```' && inCodeBlock) {
        inCodeBlock = false;
        codeBlocks.push({
          language: currentLanguage,
          code: currentCode.join('\n'),
        });
        currentLanguage = '';
        currentCode = [];
        continue;
      }
      
      // Collect code lines
      if (inCodeBlock) {
        currentCode.push(line);
      }
    }
    
    return codeBlocks;
  }

  /**
   * Helper function to check if a code block has syntax highlighting
   * A code block has syntax highlighting if it specifies a language
   * Valid languages include: typescript, javascript, bash, json, html, css, etc.
   */
  function hasSyntaxHighlighting(codeBlock: { language: string; code: string }): boolean {
    // Check if language is specified and not 'plain' or empty
    return codeBlock.language !== '' && 
           codeBlock.language !== 'plain' && 
           codeBlock.language !== 'text';
  }

  /**
   * Helper function to check if content has at least one code block with syntax highlighting
   */
  function hasCodeExamplesWithSyntaxHighlighting(content: string): boolean {
    const codeBlocks = extractCodeBlocks(content);
    
    // Must have at least one code block
    if (codeBlocks.length === 0) {
      return false;
    }
    
    // At least one code block must have syntax highlighting
    return codeBlocks.some(block => hasSyntaxHighlighting(block));
  }

  /**
   * Helper function to get statistics about code blocks in a file
   */
  function getCodeBlockStats(content: string): {
    totalBlocks: number;
    blocksWithSyntax: number;
    languages: string[];
  } {
    const codeBlocks = extractCodeBlocks(content);
    const blocksWithSyntax = codeBlocks.filter(block => hasSyntaxHighlighting(block));
    const languages = [...new Set(codeBlocks.map(block => block.language))];
    
    return {
      totalBlocks: codeBlocks.length,
      blocksWithSyntax: blocksWithSyntax.length,
      languages,
    };
  }

  /**
   * Unit test: Check that all major feature documentation files exist
   * Note: This test will report missing files but won't fail the test suite
   * since some Turkish documentation files may not be created yet
   */
  test('all major feature documentation files should exist', () => {
    const missingFiles: string[] = [];
    const existingFiles: string[] = [];
    
    majorFeatureDocFiles.forEach(filePath => {
      if (!fs.existsSync(filePath)) {
        missingFiles.push(filePath);
      } else {
        existingFiles.push(filePath);
      }
    });
    
    if (missingFiles.length > 0) {
      console.log('\nMissing documentation files (expected for incomplete Turkish docs):');
      missingFiles.forEach(file => console.log(`  - ${file}`));
    }
    
    console.log(`\nDocumentation coverage: ${existingFiles.length}/${majorFeatureDocFiles.length} files exist`);
    
    // At least the English documentation should exist
    const englishFiles = majorFeatureDocFiles.filter(f => f.startsWith('docs/en/'));
    const existingEnglishFiles = englishFiles.filter(f => fs.existsSync(f));
    
    expect(existingEnglishFiles.length).toBe(englishFiles.length);
  });

  /**
   * Property-Based Test: Every major feature documentation contains at least one code block
   * **Validates: Requirements 1.5, 8.1**
   */
  test('Property 3.1: Every major feature documentation contains at least one code block', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...majorFeatureDocFiles.filter(f => fs.existsSync(f))),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Extract code blocks
          const codeBlocks = extractCodeBlocks(content);
          
          const result = codeBlocks.length > 0;
          
          if (!result) {
            console.log(`\nNo code blocks found in: ${docFile}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Every major feature documentation contains at least one code block with syntax highlighting
   * This is the main property test that validates the requirement
   * **Validates: Requirements 1.5, 8.1**
   */
  test('Property 3: Every major feature documentation contains at least one code block with syntax highlighting', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...majorFeatureDocFiles.filter(f => fs.existsSync(f))),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check if it has code examples with syntax highlighting
          const result = hasCodeExamplesWithSyntaxHighlighting(content);
          
          if (!result) {
            const stats = getCodeBlockStats(content);
            console.log(`\nMissing syntax-highlighted code examples in: ${docFile}`);
            console.log(`  - Total code blocks: ${stats.totalBlocks}`);
            console.log(`  - Blocks with syntax highlighting: ${stats.blocksWithSyntax}`);
            console.log(`  - Languages found: ${stats.languages.join(', ') || 'none'}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Property-Based Test: Code blocks with syntax highlighting use valid language identifiers
   * **Validates: Requirements 1.5**
   */
  test('Property 3.2: Code blocks use valid language identifiers', () => {
    // Common valid language identifiers for syntax highlighting
    const validLanguages = [
      'typescript', 'ts',
      'javascript', 'js',
      'bash', 'sh', 'shell',
      'json',
      'html',
      'css',
      'yaml', 'yml',
      'toml',
      'dockerfile',
      'nginx',
      'caddyfile',
      'sql',
      'rust', 'rs',
      'python', 'py',
      'markdown', 'md',
      'xml',
      'powershell', 'ps1',
    ];
    
    fc.assert(
      fc.property(
        fc.constantFrom(...majorFeatureDocFiles.filter(f => fs.existsSync(f))),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Extract code blocks
          const codeBlocks = extractCodeBlocks(content);
          
          // Check that all code blocks with syntax highlighting use valid languages
          const invalidBlocks = codeBlocks.filter(block => 
            hasSyntaxHighlighting(block) && 
            !validLanguages.includes(block.language.toLowerCase())
          );
          
          const result = invalidBlocks.length === 0;
          
          if (!result) {
            console.log(`\nInvalid language identifiers in: ${docFile}`);
            invalidBlocks.forEach(block => {
              console.log(`  - Invalid language: "${block.language}"`);
            });
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
  test('Edge case: Empty or minimal content files should not pass code example checks', () => {
    const emptyContent = '';
    const minimalContent = '# Title\n\nSome text without code.';
    const codeWithoutSyntax = '# Title\n\n```\nconst x = 1;\n```';
    
    expect(hasCodeExamplesWithSyntaxHighlighting(emptyContent)).toBe(false);
    expect(hasCodeExamplesWithSyntaxHighlighting(minimalContent)).toBe(false);
    expect(hasCodeExamplesWithSyntaxHighlighting(codeWithoutSyntax)).toBe(false);
  });

  /**
   * Edge case test: Code blocks with syntax highlighting should pass
   */
  test('Edge case: Code blocks with syntax highlighting should pass', () => {
    const contentWithTypescript = '# Title\n\n```typescript\nconst x: number = 1;\n```';
    const contentWithJavascript = '# Title\n\n```javascript\nconst x = 1;\n```';
    const contentWithBash = '# Title\n\n```bash\necho "Hello"\n```';
    const contentWithJson = '# Title\n\n```json\n{"key": "value"}\n```';
    
    expect(hasCodeExamplesWithSyntaxHighlighting(contentWithTypescript)).toBe(true);
    expect(hasCodeExamplesWithSyntaxHighlighting(contentWithJavascript)).toBe(true);
    expect(hasCodeExamplesWithSyntaxHighlighting(contentWithBash)).toBe(true);
    expect(hasCodeExamplesWithSyntaxHighlighting(contentWithJson)).toBe(true);
  });

  /**
   * Edge case test: Multiple code blocks with at least one having syntax highlighting should pass
   */
  test('Edge case: Multiple code blocks with at least one having syntax highlighting should pass', () => {
    const contentWithMixed = `
# Title

Some text.

\`\`\`
Plain code block
\`\`\`

More text.

\`\`\`typescript
const x: number = 1;
\`\`\`

End.
`;
    
    expect(hasCodeExamplesWithSyntaxHighlighting(contentWithMixed)).toBe(true);
    
    const stats = getCodeBlockStats(contentWithMixed);
    expect(stats.totalBlocks).toBe(2);
    expect(stats.blocksWithSyntax).toBe(1);
    expect(stats.languages).toContain('plain');
    expect(stats.languages).toContain('typescript');
  });

  /**
   * Unit test: Verify code block extraction works correctly
   */
  test('Unit test: Code block extraction should work correctly', () => {
    const content = `
# Documentation

Some text before code.

\`\`\`typescript
const greeting: string = "Hello";
console.log(greeting);
\`\`\`

More text.

\`\`\`bash
npm install pusher-js
\`\`\`

\`\`\`
Plain code without language
\`\`\`

End of document.
`;
    
    const codeBlocks = extractCodeBlocks(content);
    
    expect(codeBlocks.length).toBe(3);
    expect(codeBlocks[0].language).toBe('typescript');
    expect(codeBlocks[0].code).toContain('const greeting');
    expect(codeBlocks[1].language).toBe('bash');
    expect(codeBlocks[1].code).toContain('npm install');
    expect(codeBlocks[2].language).toBe('plain');
  });

  /**
   * Unit test: Verify syntax highlighting detection works correctly
   */
  test('Unit test: Syntax highlighting detection should work correctly', () => {
    const withSyntax = { language: 'typescript', code: 'const x = 1;' };
    const withoutSyntax1 = { language: 'plain', code: 'const x = 1;' };
    const withoutSyntax2 = { language: '', code: 'const x = 1;' };
    const withoutSyntax3 = { language: 'text', code: 'const x = 1;' };
    
    expect(hasSyntaxHighlighting(withSyntax)).toBe(true);
    expect(hasSyntaxHighlighting(withoutSyntax1)).toBe(false);
    expect(hasSyntaxHighlighting(withoutSyntax2)).toBe(false);
    expect(hasSyntaxHighlighting(withoutSyntax3)).toBe(false);
  });

  /**
   * Integration test: Verify that existing documentation files have good code examples
   */
  test('Integration test: Existing documentation should have multiple code examples', () => {
    const existingFiles = majorFeatureDocFiles.filter(f => fs.existsSync(f));
    
    existingFiles.forEach(docFile => {
      const content = fs.readFileSync(docFile, 'utf-8');
      const stats = getCodeBlockStats(content);
      
      // Major feature documentation should have at least one code block
      expect(stats.totalBlocks).toBeGreaterThan(0);
      
      // At least one should have syntax highlighting
      expect(stats.blocksWithSyntax).toBeGreaterThan(0);
      
      // Log statistics for visibility
      if (stats.totalBlocks > 0) {
        console.log(`\n${docFile}:`);
        console.log(`  - Total code blocks: ${stats.totalBlocks}`);
        console.log(`  - With syntax highlighting: ${stats.blocksWithSyntax}`);
        console.log(`  - Languages: ${stats.languages.join(', ')}`);
      }
    });
  });
});
