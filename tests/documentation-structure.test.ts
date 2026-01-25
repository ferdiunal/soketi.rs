/**
 * Property-Based Tests for Documentation Structure Integrity
 * Feature: documentation-and-deployment-enhancements
 * Property 2: Dokümantasyon Yapısı Bütünlüğü
 * 
 * **Validates: Requirements 1.1, 7.1, 7.3, 7.5, 7.6**
 * 
 * Tests that every main documentation file contains:
 * - Table of contents
 * - Clear headings
 * - Consistent markdown formatting
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 2: Documentation Structure Integrity', () => {
  // Define main documentation files to test
  const mainDocFiles = [
    'docs/en/getting-started.md',
    'docs/en/installation.md',
    'docs/en/configuration.md',
    'docs/en/api-reference.md',
    'docs/tr/baslangic.md',
    'docs/tr/kurulum.md',
    'docs/tr/yapilandirma.md',
    'docs/tr/api-referans.md',
  ];

  /**
   * Helper function to check if a file contains a table of contents
   * TOC can be in various formats:
   * - ## Contents / ## İçindekiler / ## Table of Contents
   * - Links with anchors like [Section](#section)
   */
  function hasTableOfContents(content: string): boolean {
    // Check for common TOC headers
    const tocHeaders = [
      /##\s*(table of contents|contents|içindekiler|i̇çindekiler)/i,
    ];
    
    const hasTocHeader = tocHeaders.some(pattern => pattern.test(content));
    
    // Check for anchor links (typical in TOC)
    const hasAnchorLinks = /\[.+\]\(#.+\)/.test(content);
    
    return hasTocHeader || hasAnchorLinks;
  }

  /**
   * Helper function to extract headings while ignoring code blocks
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
      if (/^#{1,6}\s+.+/.test(line)) {
        headings.push(line);
      }
    }
    
    return headings;
  }

  /**
   * Helper function to check if a file has clear headings
   * Clear headings means:
   * - At least one H1 heading (# Title)
   * - Multiple H2 or H3 headings for structure
   */
  function hasClearHeadings(content: string): boolean {
    const headings = extractHeadings(content);
    
    // Check for H1 heading
    const hasH1 = headings.some(line => /^#\s+.+/.test(line));
    
    // Check for H2 or H3 headings
    const h2h3Headings = headings.filter(line => /^#{2,3}\s+.+/.test(line));
    
    // Should have at least one H1 and at least 2 H2/H3 headings for structure
    return hasH1 && h2h3Headings.length >= 2;
  }

  /**
   * Helper function to check consistent markdown formatting
   * Checks for:
   * - Proper heading hierarchy (no skipping levels)
   * - Code blocks are properly closed
   * - Lists are properly formatted
   */
  function hasConsistentMarkdownFormatting(content: string): boolean {
    const lines = content.split('\n');
    
    // Extract headings while ignoring code blocks
    const headings = extractHeadings(content)
      .map(line => {
        const match = line.match(/^(#{1,6})\s+/);
        return match ? match[1].length : 0;
      });
    
    // Check if headings don't skip levels (e.g., H1 -> H3 without H2)
    let validHierarchy = true;
    for (let i = 1; i < headings.length; i++) {
      if (headings[i] > headings[i - 1] + 1) {
        validHierarchy = false;
        break;
      }
    }
    
    // Check code blocks are properly closed
    const codeBlockMarkers = content.match(/```/g);
    const codeBlocksClosed = !codeBlockMarkers || codeBlockMarkers.length % 2 === 0;
    
    // Check for basic list formatting (lines starting with - or * or numbers)
    // But skip lines inside code blocks
    let inCodeBlock = false;
    const listItems = [];
    for (const line of lines) {
      if (line.trim().startsWith('```')) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      if (!inCodeBlock && (/^\s*[-*]\s+.+/.test(line) || /^\s*\d+\.\s+.+/.test(line))) {
        listItems.push(line);
      }
    }
    
    const hasProperLists = listItems.length === 0 || listItems.every(line => {
      // List items should have content after the marker
      return /^\s*[-*]\s+.{2,}/.test(line) || /^\s*\d+\.\s+.{2,}/.test(line);
    });
    
    return validHierarchy && codeBlocksClosed && hasProperLists;
  }

  /**
   * Unit test: Check that all main documentation files exist
   */
  test('all main documentation files should exist', () => {
    mainDocFiles.forEach(filePath => {
      expect(fs.existsSync(filePath)).toBe(true);
    });
  });

  /**
   * Property-Based Test: Every main documentation file contains table of contents
   * **Validates: Requirements 1.1, 7.3**
   */
  test('Property 2.1: Every main documentation file contains table of contents', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...mainDocFiles),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check if it has a table of contents
          const result = hasTableOfContents(content);
          
          if (!result) {
            console.log(`Missing TOC in: ${docFile}`);
          }
          
          return result;
        }
      ),
      { numRuns: mainDocFiles.length }
    );
  });

  /**
   * Property-Based Test: Every main documentation file has clear headings
   * **Validates: Requirements 7.5, 7.6**
   */
  test('Property 2.2: Every main documentation file has clear headings', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...mainDocFiles),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check if it has clear headings
          const result = hasClearHeadings(content);
          
          if (!result) {
            console.log(`Missing clear headings in: ${docFile}`);
          }
          
          return result;
        }
      ),
      { numRuns: mainDocFiles.length }
    );
  });

  /**
   * Property-Based Test: Every main documentation file has consistent markdown formatting
   * **Validates: Requirements 7.1, 7.5**
   */
  test('Property 2.3: Every main documentation file has consistent markdown formatting', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...mainDocFiles),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check if it has consistent markdown formatting
          const result = hasConsistentMarkdownFormatting(content);
          
          if (!result) {
            console.log(`Inconsistent markdown formatting in: ${docFile}`);
          }
          
          return result;
        }
      ),
      { numRuns: mainDocFiles.length }
    );
  });

  /**
   * Property-Based Test: Combined - Every main documentation file has complete structure
   * This is the main property test that validates all aspects together
   * **Validates: Requirements 1.1, 7.1, 7.3, 7.5, 7.6**
   */
  test('Property 2: Every main documentation file has complete structure integrity', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...mainDocFiles),
        (docFile) => {
          // Read the file content
          const content = fs.readFileSync(docFile, 'utf-8');
          
          // Check all three aspects
          const hasTOC = hasTableOfContents(content);
          const hasHeadings = hasClearHeadings(content);
          const hasFormatting = hasConsistentMarkdownFormatting(content);
          
          const result = hasTOC && hasHeadings && hasFormatting;
          
          if (!result) {
            console.log(`\nStructure integrity issues in: ${docFile}`);
            console.log(`  - Has TOC: ${hasTOC}`);
            console.log(`  - Has clear headings: ${hasHeadings}`);
            console.log(`  - Has consistent formatting: ${hasFormatting}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 } // Run 100 iterations as specified in design doc
    );
  });

  /**
   * Edge case test: Empty or very short files should fail the property
   */
  test('Edge case: Empty or minimal content files should not pass structure checks', () => {
    const emptyContent = '';
    const minimalContent = '# Title\n\nSome text.';
    
    expect(hasTableOfContents(emptyContent)).toBe(false);
    expect(hasClearHeadings(emptyContent)).toBe(false);
    expect(hasConsistentMarkdownFormatting(emptyContent)).toBe(true); // Empty is technically consistent
    
    expect(hasTableOfContents(minimalContent)).toBe(false);
    expect(hasClearHeadings(minimalContent)).toBe(false); // Needs at least 2 H2/H3 headings
  });

  /**
   * Edge case test: Malformed markdown should fail formatting check
   */
  test('Edge case: Malformed markdown should fail formatting check', () => {
    const unclosedCodeBlock = '# Title\n\n```typescript\nconst x = 1;\n\nNo closing backticks';
    const skippedHeadingLevel = '# Title\n\n### Subsection\n\nSkipped H2';
    
    expect(hasConsistentMarkdownFormatting(unclosedCodeBlock)).toBe(false);
    expect(hasConsistentMarkdownFormatting(skippedHeadingLevel)).toBe(false);
  });
});
