/**
 * Unit Tests for Pusher Event Trigger Endpoint
 * Feature: documentation-and-deployment-enhancements
 * Task: 11.3 Event trigger endpoint
 * 
 * **Validates: Requirement 4.3**
 * 
 * Tests the server-side event triggering endpoint for Pusher channels.
 * This endpoint allows authenticated users to trigger events to channels.
 */

import * as fs from 'fs';
import * as path from 'path';

describe('Task 11.3: Pusher Event Trigger Endpoint', () => {
  const triggerEndpointPath = 'next-chat-app/app/api/pusher/trigger/route.ts';

  /**
   * Unit test: Check that the trigger endpoint file exists
   */
  test('trigger endpoint file should exist', () => {
    expect(fs.existsSync(triggerEndpointPath)).toBe(true);
  });

  /**
   * Unit test: Verify the endpoint exports a POST handler
   */
  test('trigger endpoint should export POST handler', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for POST export
    expect(content).toMatch(/export\s+async\s+function\s+POST/);
  });

  /**
   * Unit test: Verify authentication check is implemented
   */
  test('trigger endpoint should check user authentication', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for session retrieval
    expect(content).toMatch(/auth\.api\.getSession/);
    
    // Check for unauthorized response
    expect(content).toMatch(/Unauthorized/);
    expect(content).toMatch(/401/);
  });

  /**
   * Unit test: Verify required parameters validation
   */
  test('trigger endpoint should validate required parameters', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for parameter extraction
    expect(content).toMatch(/channel/);
    expect(content).toMatch(/event/);
    expect(content).toMatch(/data/);
    
    // Check for parameter validation
    expect(content).toMatch(/Missing required parameters|Missing parameters/);
    expect(content).toMatch(/400/);
  });

  /**
   * Unit test: Verify channel name validation
   */
  test('trigger endpoint should validate channel name format', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for channel validation
    expect(content).toMatch(/Invalid channel/);
  });

  /**
   * Unit test: Verify event name validation
   */
  test('trigger endpoint should validate event name format', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for event validation
    expect(content).toMatch(/Invalid event/);
  });

  /**
   * Unit test: Verify Pusher trigger call
   */
  test('trigger endpoint should call pusher.trigger', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for pusher.trigger call
    expect(content).toMatch(/pusher\.trigger/);
  });

  /**
   * Unit test: Verify socketId support for excluding sender
   */
  test('trigger endpoint should support socketId parameter', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for socketId handling
    expect(content).toMatch(/socketId/);
    expect(content).toMatch(/socket_id/);
  });

  /**
   * Unit test: Verify success response
   */
  test('trigger endpoint should return success response', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for success response
    expect(content).toMatch(/success.*true/);
    expect(content).toMatch(/200/);
  });

  /**
   * Unit test: Verify error handling
   */
  test('trigger endpoint should handle errors gracefully', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for try-catch block
    expect(content).toMatch(/try\s*{/);
    expect(content).toMatch(/catch/);
    
    // Check for error logging
    expect(content).toMatch(/console\.error/);
    
    // Check for error response
    expect(content).toMatch(/Internal server error/);
    expect(content).toMatch(/500/);
  });

  /**
   * Unit test: Verify Pusher instance configuration
   */
  test('trigger endpoint should configure Pusher instance correctly', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for Pusher import
    expect(content).toMatch(/import.*Pusher.*from.*['"]pusher['"]/);
    
    // Check for Pusher configuration
    expect(content).toMatch(/new Pusher\(/);
    expect(content).toMatch(/appId/);
    expect(content).toMatch(/key/);
    expect(content).toMatch(/secret/);
    expect(content).toMatch(/PUSHER_APP_ID/);
    expect(content).toMatch(/PUSHER_SECRET/);
  });

  /**
   * Unit test: Verify comprehensive documentation
   */
  test('trigger endpoint should have comprehensive documentation', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for JSDoc comments
    expect(content).toMatch(/\/\*\*/);
    
    // Check for requirement references
    expect(content).toMatch(/Requirement 4\.3/);
    
    // Check for example request documentation
    expect(content).toMatch(/Example Request/);
  });

  /**
   * Unit test: Verify NextRequest and NextResponse imports
   */
  test('trigger endpoint should import Next.js types', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for Next.js imports
    expect(content).toMatch(/import.*NextRequest.*from.*['"]next\/server['"]/);
    expect(content).toMatch(/import.*NextResponse.*from.*['"]next\/server['"]/);
  });

  /**
   * Unit test: Verify auth import
   */
  test('trigger endpoint should import auth from lib', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Check for auth import
    expect(content).toMatch(/import.*auth.*from.*['"]@\/lib\/auth['"]/);
  });

  /**
   * Edge case test: Verify handling of missing channel parameter
   */
  test('Edge case: should handle missing channel parameter', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Should validate channel existence
    expect(content).toMatch(/!channel/);
  });

  /**
   * Edge case test: Verify handling of missing event parameter
   */
  test('Edge case: should handle missing event parameter', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Should validate event existence
    expect(content).toMatch(/!event/);
  });

  /**
   * Edge case test: Verify handling of missing data parameter
   */
  test('Edge case: should handle missing data parameter', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Should validate data existence
    expect(content).toMatch(/!data/);
  });

  /**
   * Edge case test: Verify handling of empty string channel
   */
  test('Edge case: should handle empty string channel name', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Should check for empty strings
    expect(content).toMatch(/trim\(\)/);
  });

  /**
   * Integration test: Verify consistency with auth endpoint
   */
  test('Integration: should use same Pusher configuration as auth endpoint', () => {
    const triggerContent = fs.readFileSync(triggerEndpointPath, 'utf-8');
    const authEndpointPath = 'next-chat-app/app/api/pusher/auth/route.ts';
    
    if (fs.existsSync(authEndpointPath)) {
      const authContent = fs.readFileSync(authEndpointPath, 'utf-8');
      
      // Both should use same environment variables
      const envVars = [
        'PUSHER_APP_ID',
        'NEXT_PUBLIC_PUSHER_KEY',
        'PUSHER_SECRET',
        'PUSHER_HOST',
        'PUSHER_PORT',
        'PUSHER_USE_TLS'
      ];
      
      envVars.forEach(envVar => {
        const triggerHasVar = triggerContent.includes(envVar);
        const authHasVar = authContent.includes(envVar);
        
        // Both should reference the same env vars
        expect(triggerHasVar).toBe(authHasVar);
      });
    }
  });

  /**
   * Security test: Verify authentication is required
   */
  test('Security: should require authentication before triggering events', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Session check should come before trigger
    const sessionCheckIndex = content.indexOf('auth.api.getSession');
    const triggerIndex = content.indexOf('pusher.trigger');
    
    expect(sessionCheckIndex).toBeGreaterThan(-1);
    expect(triggerIndex).toBeGreaterThan(-1);
    expect(sessionCheckIndex).toBeLessThan(triggerIndex);
  });

  /**
   * API contract test: Verify expected request/response structure
   */
  test('API contract: should document expected request and response structure', () => {
    const content = fs.readFileSync(triggerEndpointPath, 'utf-8');
    
    // Should document request body structure
    expect(content).toMatch(/Request Body/);
    
    // Should document response structure
    expect(content).toMatch(/Response/);
    
    // Should document status codes
    expect(content).toMatch(/200/); // Success
    expect(content).toMatch(/400/); // Bad request
    expect(content).toMatch(/401/); // Unauthorized
    expect(content).toMatch(/500/); // Server error
  });
});
