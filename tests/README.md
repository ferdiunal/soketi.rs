# Documentation and Deployment Tests

This directory contains property-based tests and unit tests for the documentation-and-deployment-enhancements feature.

## Test Framework

- **Testing Framework**: Jest with TypeScript support (ts-jest)
- **Property-Based Testing**: fast-check library
- **Minimum Iterations**: 100 runs per property test (as specified in design document)

## Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm test -- tests/documentation-structure.test.ts
```

## Test Files

### documentation-structure.test.ts

**Property 2: Dokümantasyon Yapısı Bütünlüğü**

Tests that every main documentation file contains:
- Table of contents
- Clear headings (H1, H2, H3 hierarchy)
- Consistent markdown formatting

**Validates Requirements**: 1.1, 7.1, 7.3, 7.5, 7.6

**Test Coverage**:
- ✅ Property 2.1: Every main documentation file contains table of contents
- ✅ Property 2.2: Every main documentation file has clear headings
- ✅ Property 2.3: Every main documentation file has consistent markdown formatting
- ✅ Property 2: Combined test for complete structure integrity (100 iterations)
- ✅ Edge cases: Empty files, minimal content, malformed markdown

**Files Tested**:
- docs/en/getting-started.md
- docs/en/installation.md
- docs/en/configuration.md
- docs/en/api-reference.md
- docs/tr/baslangic.md
- docs/tr/kurulum.md
- docs/tr/yapilandirma.md
- docs/tr/api-referans.md

## Test Implementation Details

### Code Block Handling

The tests properly handle markdown code blocks to avoid false positives:
- Ignores lines inside code blocks when checking for headings
- Prevents bash comments (starting with `#`) from being detected as markdown headings
- Validates that code blocks are properly closed (matching ``` pairs)

### Heading Hierarchy Validation

The tests validate proper markdown heading hierarchy:
- Ensures H1 → H2 → H3 progression (no skipping levels)
- Requires at least one H1 heading
- Requires at least 2 H2/H3 headings for proper structure

### Table of Contents Detection

The tests detect TOC in multiple formats:
- Common headers: "Table of Contents", "Contents", "İçindekiler"
- Anchor links: `[Section](#section)` format

### List Formatting

The tests validate proper list formatting:
- Checks for proper bullet points (`-` or `*`)
- Checks for proper numbered lists (`1.`, `2.`, etc.)
- Ensures list items have meaningful content (at least 2 characters)

## Property-Based Testing Approach

Following the design document specifications:
- Each property test runs with minimum 100 iterations
- Tests use `fc.constantFrom()` to test all documentation files
- Tests include both individual property checks and combined integrity checks
- Edge cases are tested separately with unit tests

## Test Results

All tests passing ✅

```
Test Suites: 1 passed, 1 total
Tests:       7 passed, 7 total
```

## Next Steps

Additional property tests to be implemented:
- Property 1: Language Consistency (Task 3.6)
- Property 3: Code Examples Presence (Task 2.6)
- Property 4: Glossary Terminology Consistency (Task 3.7)
- Property 5-6: README Section Presence and Links (Task 4.3)
- Property 7: Reverse Proxy Configuration Correctness (Task 7.5)
- Property 8-9: Channel Type Support and Connection Status (Tasks 11.4, 12.5)
- Property 10: Deployment Documentation Content (Task 17.3)
- Property 11: Build Optimization Configuration (Task 15.6)
- Property 12: Documentation Cross-References (Task 19.2)
- Property 13-14: Multi-language Code Examples and Example Documentation (Tasks 18.3, 18.4)
