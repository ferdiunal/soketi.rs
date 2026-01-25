/**
 * Property-Based Tests for Glossary Terminology Consistency
 * Feature: documentation-and-deployment-enhancements
 * Property 4: Sözlük Terminoloji Tutarlılığı
 * 
 * **Validates: Requirements 1.4**
 * 
 * Tests that for every documentation file, terms defined in the glossary
 * must be used consistently (in terms of capitalization and format).
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 4: Glossary Terminology Consistency', () => {
  /**
   * Glossary terms from requirements.md (Sözlük section)
   * These terms should be used consistently across all documentation
   * 
   * Note: Both underscore format (Soketi_Server) and space format (Soketi Server)
   * are considered valid and equivalent for readability in documentation.
   */
  const glossaryTerms: Record<string, string[]> = {
    'Soketi_Server': ['Soketi_Server', 'Soketi Server'],
    'Documentation_System': ['Documentation_System', 'Documentation System'],
    'Reverse_Proxy': ['Reverse_Proxy', 'Reverse Proxy'],
    'Next_Chat_App': ['Next_Chat_App', 'Next Chat App'],
    'Deployment_Platform': ['Deployment_Platform', 'Deployment Platform'],
    'HTTP2': ['HTTP2', 'HTTP/2'],
    'HTTP3': ['HTTP3', 'HTTP/3'],
    'Better_Auth': ['Better_Auth', 'Better Auth'],
    'Pusher_SDK': ['Pusher_SDK', 'Pusher SDK'],
  };

  /**
   * Alternative forms that should NOT be used (inconsistent variations)
   * These are common mistakes or inconsistent capitalizations
   * 
   * Note: Lowercase usage in the middle of sentences (e.g., "reverse proxy")
   * is acceptable as a generic term. We only flag clearly incorrect forms.
   */
  const inconsistentVariations: Record<string, string[]> = {
    'Soketi_Server': ['soketi_server', 'Soketi_server', 'soketi server'],
    'Documentation_System': ['documentation_system', 'Documentation_system', 'documentation system'],
    'Reverse_Proxy': ['reverse_proxy', 'Reverse_proxy'],
    'Next_Chat_App': ['next_chat_app', 'Next_chat_app', 'next chat app'],
    'Deployment_Platform': ['deployment_platform', 'Deployment_platform', 'deployment platform'],
    'HTTP2': ['http2', 'Http2'],
    'HTTP3': ['http3', 'Http3'],
    'Better_Auth': ['better_auth', 'Better_auth', 'better auth'],
    'Pusher_SDK': ['pusher_sdk', 'Pusher_sdk', 'pusher sdk'],
  };

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
   * Helper function to find all occurrences of a term in content
   * Returns array of { term, line, column, context }
   */
  function findTermOccurrences(
    content: string,
    term: string
  ): Array<{ term: string; line: number; column: number; context: string }> {
    const occurrences: Array<{ term: string; line: number; column: number; context: string }> = [];
    const lines = content.split('\n');
    
    let inCodeBlock = false;
    
    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
      const line = lines[lineIndex];
      
      // Toggle code block state
      if (line.trim().startsWith('```')) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      
      // Skip lines inside code blocks (code examples can use different formats)
      if (inCodeBlock) {
        continue;
      }
      
      // Find all occurrences of the term in this line
      let columnIndex = 0;
      while (columnIndex < line.length) {
        const index = line.indexOf(term, columnIndex);
        if (index === -1) break;
        
        // Check if this is a whole word match (not part of another word)
        const beforeChar = index > 0 ? line[index - 1] : ' ';
        const afterChar = index + term.length < line.length ? line[index + term.length] : ' ';
        
        // Allow word boundaries: space, punctuation, start/end of line
        const isWordBoundary = (char: string) => /[\s.,;:!?()\[\]{}"'`]/.test(char);
        
        if (isWordBoundary(beforeChar) && isWordBoundary(afterChar)) {
          occurrences.push({
            term,
            line: lineIndex + 1,
            column: index + 1,
            context: line.trim(),
          });
        }
        
        columnIndex = index + 1;
      }
    }
    
    return occurrences;
  }

  /**
   * Helper function to check if content uses glossary terms consistently
   * Returns { isConsistent, violations }
   */
  function checkTerminologyConsistency(
    content: string,
    filePath: string
  ): {
    isConsistent: boolean;
    violations: Array<{
      incorrectTerm: string;
      correctTerm: string;
      occurrences: Array<{ line: number; column: number; context: string }>;
    }>;
  } {
    const violations: Array<{
      incorrectTerm: string;
      correctTerm: string;
      occurrences: Array<{ line: number; column: number; context: string }>;
    }> = [];

    // Check for inconsistent variations of each glossary term
    for (const [correctTerm, variations] of Object.entries(inconsistentVariations)) {
      for (const incorrectTerm of variations) {
        const occurrences = findTermOccurrences(content, incorrectTerm);
        
        if (occurrences.length > 0) {
          violations.push({
            incorrectTerm,
            correctTerm,
            occurrences: occurrences.map(occ => ({
              line: occ.line,
              column: occ.column,
              context: occ.context,
            })),
          });
        }
      }
    }

    return {
      isConsistent: violations.length === 0,
      violations,
    };
  }

  /**
   * Helper function to get all documentation files
   */
  function getAllDocumentationFiles(): string[] {
    const docsDir = 'docs';
    const files: string[] = [];
    
    if (fs.existsSync(docsDir)) {
      const allFiles = getMarkdownFiles(docsDir, docsDir);
      files.push(...allFiles.map(f => path.join(docsDir, f)));
    }
    
    // Also check README files
    if (fs.existsSync('README.md')) {
      files.push('README.md');
    }
    if (fs.existsSync('README.tr.md')) {
      files.push('README.tr.md');
    }
    
    return files;
  }

  // Get all documentation files
  const documentationFiles = getAllDocumentationFiles();

  /**
   * Unit test: Check that documentation files exist
   */
  test('documentation files should exist', () => {
    expect(documentationFiles.length).toBeGreaterThan(0);
  });

  /**
   * Unit test: Check that glossary terms are defined
   */
  test('glossary terms should be defined', () => {
    expect(Object.keys(glossaryTerms).length).toBeGreaterThan(0);
  });

  /**
   * Property-Based Test: For every documentation file, glossary terms are used consistently
   * This is the main property test that validates the terminology consistency requirement
   * **Validates: Requirements 1.4**
   */
  test('Property 4: For every documentation file, glossary terms are used consistently', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...documentationFiles),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check terminology consistency
          const { isConsistent, violations } = checkTerminologyConsistency(content, docFile);
          
          if (!isConsistent) {
            console.log(`\nTerminology inconsistencies found in: ${docFile}`);
            violations.forEach(violation => {
              console.log(`\n  Incorrect term: "${violation.incorrectTerm}" (should be "${violation.correctTerm}")`);
              console.log(`  Found ${violation.occurrences.length} occurrence(s):`);
              violation.occurrences.forEach(occ => {
                console.log(`    - Line ${occ.line}, Column ${occ.column}: ${occ.context}`);
              });
            });
          }
          
          return isConsistent;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Property-Based Test: Glossary terms should appear in their correct form when used
   * **Validates: Requirements 1.4**
   */
  test('Property 4.1: When glossary terms are used, they use the correct capitalization and format', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...documentationFiles),
        fc.constantFrom(...Object.keys(glossaryTerms)),
        (docFile, correctTerm) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check if any incorrect variations are used
          const variations = inconsistentVariations[correctTerm] || [];
          
          for (const incorrectTerm of variations) {
            const occurrences = findTermOccurrences(content, incorrectTerm);
            
            if (occurrences.length > 0) {
              console.log(`\nIncorrect term usage in: ${docFile}`);
              console.log(`  Found "${incorrectTerm}" instead of "${correctTerm}"`);
              console.log(`  Occurrences: ${occurrences.length}`);
              occurrences.forEach(occ => {
                console.log(`    - Line ${occ.line}: ${occ.context}`);
              });
              return false;
            }
          }
          
          return true;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Edge case test: Empty files should pass (no terms to check)
   */
  test('Edge case: Empty files should pass consistency check', () => {
    const emptyContent = '';
    const { isConsistent, violations } = checkTerminologyConsistency(emptyContent, 'test.md');
    
    expect(isConsistent).toBe(true);
    expect(violations.length).toBe(0);
  });

  /**
   * Edge case test: Files with correct terms should pass
   */
  test('Edge case: Files with correct glossary terms should pass', () => {
    const correctContent = `
# Documentation

This is about Soketi_Server and Documentation_System.

The Reverse_Proxy handles HTTP2 and HTTP3 connections.

Next_Chat_App uses Better_Auth and Pusher_SDK.

Deploy to Deployment_Platform.
`;
    
    const { isConsistent, violations } = checkTerminologyConsistency(correctContent, 'test.md');
    
    expect(isConsistent).toBe(true);
    expect(violations.length).toBe(0);
    
    // Also test with space format (should also pass)
    const correctContentWithSpaces = `
# Documentation

This is about Soketi Server and Documentation System.

The Reverse Proxy handles HTTP/2 and HTTP/3 connections.

Next Chat App uses Better Auth and Pusher SDK.

Deploy to Deployment Platform.
`;
    
    const result2 = checkTerminologyConsistency(correctContentWithSpaces, 'test.md');
    
    expect(result2.isConsistent).toBe(true);
    expect(result2.violations.length).toBe(0);
  });

  /**
   * Edge case test: Files with incorrect terms should fail
   */
  test('Edge case: Files with incorrect glossary terms should fail', () => {
    const incorrectContent = `
# Documentation

This is about soketi_server and documentation_system.

The reverse_proxy handles http2 and http3 connections.
`;
    
    const { isConsistent, violations } = checkTerminologyConsistency(incorrectContent, 'test.md');
    
    expect(isConsistent).toBe(false);
    expect(violations.length).toBeGreaterThan(0);
    
    // Check that violations are detected
    const violationTerms = violations.map(v => v.incorrectTerm);
    expect(violationTerms).toContain('soketi_server');
    expect(violationTerms).toContain('documentation_system');
    expect(violationTerms).toContain('reverse_proxy');
  });

  /**
   * Edge case test: Terms in code blocks should be ignored
   */
  test('Edge case: Terms in code blocks should be ignored', () => {
    const contentWithCodeBlock = `
# Documentation

This is about Soketi_Server.

\`\`\`typescript
// In code, we can use different formats
const server = 'soketi_server';
const proxy = 'reverse_proxy';
\`\`\`

Back to documentation with Reverse_Proxy.
`;
    
    const { isConsistent, violations } = checkTerminologyConsistency(contentWithCodeBlock, 'test.md');
    
    // Should pass because incorrect terms are in code blocks
    expect(isConsistent).toBe(true);
    expect(violations.length).toBe(0);
  });

  /**
   * Edge case test: Terms as part of other words should not be detected
   */
  test('Edge case: Terms as part of other words should not be detected', () => {
    const content = `
# Documentation

This is about Soketi_Server_Configuration and reverse_proxy_settings.
`;
    
    const { isConsistent, violations } = checkTerminologyConsistency(content, 'test.md');
    
    // Should pass because these are compound words, not the glossary terms themselves
    expect(isConsistent).toBe(true);
  });

  /**
   * Edge case test: Multiple occurrences of the same incorrect term should be reported
   */
  test('Edge case: Multiple occurrences of incorrect terms should be reported', () => {
    const content = `
# Documentation

First mention of soketi_server.

Second mention of soketi_server.

Third mention of soketi_server.
`;
    
    const { isConsistent, violations } = checkTerminologyConsistency(content, 'test.md');
    
    expect(isConsistent).toBe(false);
    expect(violations.length).toBeGreaterThan(0);
    
    const violation = violations.find(v => v.incorrectTerm === 'soketi_server');
    expect(violation).toBeDefined();
    expect(violation!.occurrences.length).toBe(3);
  });

  /**
   * Unit test: Verify term occurrence detection works correctly
   */
  test('Unit test: Term occurrence detection should work correctly', () => {
    const content = `
Line 1: This mentions Soketi_Server.
Line 2: Another Soketi_Server mention.
Line 3: No mention here.
Line 4: Final Soketi_Server.
`;
    
    const occurrences = findTermOccurrences(content, 'Soketi_Server');
    
    expect(occurrences.length).toBe(3);
    expect(occurrences[0].line).toBe(2); // Line 1 in content (after empty first line)
    expect(occurrences[1].line).toBe(3);
    expect(occurrences[2].line).toBe(5);
  });

  /**
   * Unit test: Verify term detection respects word boundaries
   */
  test('Unit test: Term detection should respect word boundaries', () => {
    const content = `
This is HTTP2 protocol.
This is HTTP2_Extended (not the glossary term).
This is myHTTP2 (not the glossary term).
This is HTTP2. (with punctuation)
`;
    
    const occurrences = findTermOccurrences(content, 'HTTP2');
    
    // Should find HTTP2 on lines 1 and 4 (with punctuation), but not in compound words
    expect(occurrences.length).toBe(2);
  });

  /**
   * Integration test: Verify that all existing documentation files use consistent terminology
   */
  test('Integration test: All existing documentation should use consistent glossary terminology', () => {
    const filesWithViolations: Array<{
      file: string;
      violationCount: number;
    }> = [];
    
    const filesWithCorrectTerminology: string[] = [];
    
    documentationFiles.forEach(docFile => {
      const content = fs.readFileSync(docFile, 'utf-8');
      const { isConsistent, violations } = checkTerminologyConsistency(content, docFile);
      
      if (!isConsistent) {
        const totalViolations = violations.reduce((sum, v) => sum + v.occurrences.length, 0);
        filesWithViolations.push({
          file: docFile,
          violationCount: totalViolations,
        });
      } else {
        filesWithCorrectTerminology.push(docFile);
      }
    });
    
    if (filesWithViolations.length > 0) {
      console.log('\nFiles with terminology violations:');
      filesWithViolations.forEach(item => {
        console.log(`  - ${item.file}: ${item.violationCount} violation(s)`);
      });
    }
    
    console.log(`\nTerminology consistency: ${filesWithCorrectTerminology.length}/${documentationFiles.length} files use consistent terminology`);
    
    // All files should use consistent terminology
    expect(filesWithViolations.length).toBe(0);
  });

  /**
   * Integration test: Verify that correct glossary terms are actually used in documentation
   */
  test('Integration test: Glossary terms should be used in documentation', () => {
    const termUsage: Record<string, number> = {};
    
    // Initialize counters
    Object.keys(glossaryTerms).forEach(term => {
      termUsage[term] = 0;
    });
    
    // Count usage across all files (checking all valid formats)
    documentationFiles.forEach(docFile => {
      const content = fs.readFileSync(docFile, 'utf-8');
      
      Object.entries(glossaryTerms).forEach(([termKey, validFormats]) => {
        // Check for any of the valid formats
        validFormats.forEach(format => {
          const occurrences = findTermOccurrences(content, format);
          termUsage[termKey] += occurrences.length;
        });
      });
    });
    
    console.log('\nGlossary term usage across all documentation:');
    Object.entries(termUsage).forEach(([term, count]) => {
      console.log(`  - ${term}: ${count} occurrence(s)`);
    });
    
    // At least some glossary terms should be used
    const totalUsage = Object.values(termUsage).reduce((sum, count) => sum + count, 0);
    expect(totalUsage).toBeGreaterThan(0);
  });
});
