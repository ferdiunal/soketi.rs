/**
 * Property-Based Tests for Channel Type Support
 * Feature: documentation-and-deployment-enhancements
 * Property 8: Kanal Tipi Desteği
 * 
 * **Validates: Requirements 4.5**
 * 
 * Tests that for every channel type (public, private, presence),
 * the Next.js chat application contains appropriate handler code.
 * 
 * Channel Types:
 * - public: No authorization required, anyone can subscribe
 * - private: Requires authorization via /api/pusher/auth endpoint
 * - presence: Requires authorization with user info via /api/pusher/auth endpoint
 */

import * as fc from 'fast-check';
import * as fs from 'fs';

describe('Property 8: Channel Type Support', () => {
  // Define the three channel types that must be supported
  const channelTypes = ['public', 'private', 'presence'] as const;
  type ChannelType = typeof channelTypes[number];

  // Define files that should contain channel type handling
  const relevantFiles = {
    authEndpoint: 'next-chat-app/app/api/pusher/auth/route.ts',
    pusherClient: 'next-chat-app/lib/pusher.ts',
    chatTypes: 'next-chat-app/types/chat.ts',
    typeIndex: 'next-chat-app/types/index.ts',
  };

  /**
   * Helper function to check if a file contains handler code for a channel type
   */
  function hasChannelTypeHandler(filePath: string, channelType: ChannelType): boolean {
    if (!fs.existsSync(filePath)) {
      return false;
    }

    const content = fs.readFileSync(filePath, 'utf-8');

    switch (channelType) {
      case 'public':
        // Public channels don't require special authorization
        // Check that the code mentions public channels in comments or documentation
        return (
          content.includes('public') &&
          (content.includes('public channel') || 
           content.includes('public-') ||
           content.includes("'public'") ||
           content.includes('"public"'))
        );

      case 'private':
        // Private channels require authorization
        // Check for private- prefix handling
        return (
          content.includes('private-') ||
          (content.includes('private') && 
           (content.includes("'private'") || content.includes('"private"')))
        );

      case 'presence':
        // Presence channels require authorization with user info
        // Check for presence- prefix handling and user info
        return (
          content.includes('presence-') ||
          (content.includes('presence') && 
           (content.includes("'presence'") || content.includes('"presence"')))
        );

      default:
        return false;
    }
  }

  /**
   * Helper function to check if auth endpoint properly handles a channel type
   */
  function authEndpointHandlesChannelType(channelType: ChannelType): boolean {
    const filePath = relevantFiles.authEndpoint;
    
    if (!fs.existsSync(filePath)) {
      return false;
    }

    const content = fs.readFileSync(filePath, 'utf-8');

    switch (channelType) {
      case 'public':
        // Public channels should be documented as not requiring authorization
        return (
          content.includes('public') &&
          (content.includes('No authorization') || 
           content.includes('not handled here') ||
           content.includes('public channels'))
        );

      case 'private':
        // Private channels should have authorization logic
        return (
          content.includes("startsWith('private-')") &&
          content.includes('authorizeChannel') &&
          // Should NOT include presenceData for private channels
          !content.match(/if\s*\([^)]*private[^)]*\)[^{]*{[^}]*presenceData/)
        );

      case 'presence':
        // Presence channels should have authorization with user info
        return (
          content.includes("startsWith('presence-')") &&
          content.includes('authorizeChannel') &&
          content.includes('presenceData') &&
          (content.includes('user_id') || content.includes('user_info'))
        );

      default:
        return false;
    }
  }

  /**
   * Helper function to check if type definitions include a channel type
   */
  function typeDefinitionsIncludeChannelType(channelType: ChannelType): boolean {
    const filePath = relevantFiles.chatTypes;
    
    if (!fs.existsSync(filePath)) {
      return false;
    }

    const content = fs.readFileSync(filePath, 'utf-8');

    // Check if ChannelType includes this type
    const channelTypeRegex = new RegExp(
      `ChannelType\\s*=\\s*[^;]*['"]${channelType}['"]`,
      's'
    );

    return channelTypeRegex.test(content);
  }

  /**
   * Helper function to check if Pusher client is configured for channel type
   */
  function pusherClientSupportsChannelType(channelType: ChannelType): boolean {
    const filePath = relevantFiles.pusherClient;
    
    if (!fs.existsSync(filePath)) {
      return false;
    }

    const content = fs.readFileSync(filePath, 'utf-8');

    switch (channelType) {
      case 'public':
        // Public channels work by default, no special config needed
        return true;

      case 'private':
      case 'presence':
        // Private and presence channels require authEndpoint
        return (
          content.includes('authEndpoint') &&
          content.includes('/api/pusher/auth')
        );

      default:
        return false;
    }
  }

  /**
   * Unit test: Check that all relevant files exist
   */
  test('all relevant files should exist', () => {
    Object.entries(relevantFiles).forEach(([name, filePath]) => {
      expect(fs.existsSync(filePath)).toBe(true);
    });
  });

  /**
   * Unit test: Check that ChannelType is properly defined
   */
  test('ChannelType should include all three channel types', () => {
    const content = fs.readFileSync(relevantFiles.chatTypes, 'utf-8');
    
    // Check for type definition
    expect(content).toMatch(/export\s+type\s+ChannelType/);
    
    // Check for all three types
    expect(content).toMatch(/'public'/);
    expect(content).toMatch(/'private'/);
    expect(content).toMatch(/'presence'/);
  });

  /**
   * Unit test: Check that Channel interface exists
   */
  test('Channel interface should exist with type property', () => {
    const content = fs.readFileSync(relevantFiles.chatTypes, 'utf-8');
    
    // Check for interface definition
    expect(content).toMatch(/export\s+interface\s+Channel/);
    
    // Check for type property
    expect(content).toMatch(/type:\s*ChannelType/);
  });

  /**
   * Unit test: Check auth endpoint handles private channels
   */
  test('auth endpoint should handle private channels', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Check for private channel handling
    expect(content).toMatch(/startsWith\(['"]private-['"]\)/);
    expect(content).toMatch(/authorizeChannel/);
  });

  /**
   * Unit test: Check auth endpoint handles presence channels
   */
  test('auth endpoint should handle presence channels with user info', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Check for presence channel handling
    expect(content).toMatch(/startsWith\(['"]presence-['"]\)/);
    expect(content).toMatch(/presenceData/);
    expect(content).toMatch(/user_id/);
    expect(content).toMatch(/user_info/);
  });

  /**
   * Unit test: Check Pusher client has auth endpoint configured
   */
  test('Pusher client should have authEndpoint configured', () => {
    const content = fs.readFileSync(relevantFiles.pusherClient, 'utf-8');
    
    // Check for authEndpoint configuration
    expect(content).toMatch(/authEndpoint:\s*['"]\/api\/pusher\/auth['"]/);
  });

  /**
   * Property-Based Test: Every channel type has handler code in auth endpoint
   * **Validates: Requirements 4.5**
   */
  test('Property 8.1: Every channel type has appropriate handler in auth endpoint', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...channelTypes),
        (channelType) => {
          const result = authEndpointHandlesChannelType(channelType);
          
          if (!result) {
            console.log(`\nMissing handler for channel type: ${channelType}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Every channel type is defined in type definitions
   * **Validates: Requirements 4.5**
   */
  test('Property 8.2: Every channel type is included in type definitions', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...channelTypes),
        (channelType) => {
          const result = typeDefinitionsIncludeChannelType(channelType);
          
          if (!result) {
            console.log(`\nMissing type definition for channel type: ${channelType}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Every channel type is supported by Pusher client
   * **Validates: Requirements 4.5**
   */
  test('Property 8.3: Every channel type is supported by Pusher client configuration', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...channelTypes),
        (channelType) => {
          const result = pusherClientSupportsChannelType(channelType);
          
          if (!result) {
            console.log(`\nPusher client doesn't support channel type: ${channelType}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Combined - Every channel type has complete support
   * This is the main property test that validates all aspects together
   * **Validates: Requirements 4.5**
   */
  test('Property 8: Every channel type has appropriate handler code in the application', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...channelTypes),
        (channelType) => {
          // Check all aspects of channel type support
          const hasAuthHandler = authEndpointHandlesChannelType(channelType);
          const hasTypeDefinition = typeDefinitionsIncludeChannelType(channelType);
          const hasPusherSupport = pusherClientSupportsChannelType(channelType);
          
          const result = hasAuthHandler && hasTypeDefinition && hasPusherSupport;
          
          if (!result) {
            console.log(`\nChannel type support issues for: ${channelType}`);
            console.log(`  - Has auth handler: ${hasAuthHandler}`);
            console.log(`  - Has type definition: ${hasTypeDefinition}`);
            console.log(`  - Has Pusher support: ${hasPusherSupport}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Edge case test: Invalid channel types should not be accepted
   */
  test('Edge case: Invalid channel types should not be in type definitions', () => {
    const content = fs.readFileSync(relevantFiles.chatTypes, 'utf-8');
    
    // These should NOT be valid channel types
    const invalidTypes = ['protected', 'secret', 'hidden', 'encrypted'];
    
    invalidTypes.forEach(invalidType => {
      // Should not be in the ChannelType definition
      const channelTypeRegex = new RegExp(
        `ChannelType\\s*=\\s*[^;]*['"]${invalidType}['"]`,
        's'
      );
      expect(channelTypeRegex.test(content)).toBe(false);
    });
  });

  /**
   * Edge case test: Auth endpoint should reject invalid channel types
   */
  test('Edge case: Auth endpoint should handle invalid channel types', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Should have error handling for invalid channel types
    expect(content).toMatch(/Invalid channel/);
  });

  /**
   * Edge case test: Private channels should not include presence data
   */
  test('Edge case: Private channels should not include presence data', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Find the private channel handling block
    const privateBlockMatch = content.match(
      /if\s*\([^)]*startsWith\(['"]private-['"]\)[^)]*\)\s*{([^}]+)}/s
    );
    
    if (privateBlockMatch) {
      const privateBlock = privateBlockMatch[1];
      // Private block should NOT contain presenceData
      expect(privateBlock).not.toMatch(/presenceData/);
    }
  });

  /**
   * Edge case test: Presence channels should include user info
   */
  test('Edge case: Presence channels should include user info', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Find the presence channel handling block
    const presenceBlockMatch = content.match(
      /if\s*\([^)]*startsWith\(['"]presence-['"]\)[^)]*\)\s*{([^}]+)}/s
    );
    
    if (presenceBlockMatch) {
      const presenceBlock = presenceBlockMatch[1];
      // Presence block MUST contain presenceData with user_id and user_info
      expect(presenceBlock).toMatch(/presenceData/);
      expect(presenceBlock).toMatch(/user_id/);
      expect(presenceBlock).toMatch(/user_info/);
    }
  });

  /**
   * Integration test: Channel types should be exported from index
   */
  test('Integration: ChannelType should be exported from types/index.ts', () => {
    const content = fs.readFileSync(relevantFiles.typeIndex, 'utf-8');
    
    // Should export ChannelType (check with multiline support)
    expect(content).toMatch(/ChannelType/);
    expect(content).toMatch(/export\s+type\s*{/);
  });

  /**
   * Integration test: Channel interface should be exported from index
   */
  test('Integration: Channel interface should be exported from types/index.ts', () => {
    const content = fs.readFileSync(relevantFiles.typeIndex, 'utf-8');
    
    // Should export Channel (check with multiline support)
    expect(content).toMatch(/Channel/);
    expect(content).toMatch(/export\s+type\s*{/);
  });

  /**
   * Security test: Auth endpoint should require authentication for private/presence
   */
  test('Security: Auth endpoint should require authentication', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Should check session before authorizing
    expect(content).toMatch(/auth\.api\.getSession/);
    expect(content).toMatch(/Unauthorized/);
    expect(content).toMatch(/401/);
    
    // Session check should come before authorization
    const sessionCheckIndex = content.indexOf('auth.api.getSession');
    const authorizeIndex = content.indexOf('authorizeChannel');
    
    expect(sessionCheckIndex).toBeGreaterThan(-1);
    expect(authorizeIndex).toBeGreaterThan(-1);
    expect(sessionCheckIndex).toBeLessThan(authorizeIndex);
  });

  /**
   * Documentation test: Channel types should be documented
   */
  test('Documentation: Channel types should be documented in auth endpoint', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // Should have documentation about channel types
    expect(content).toMatch(/Channel Types/);
    expect(content).toMatch(/private-\*/);
    expect(content).toMatch(/presence-\*/);
    expect(content).toMatch(/public/);
  });

  /**
   * Documentation test: Type definitions should have JSDoc comments
   */
  test('Documentation: ChannelType should have JSDoc documentation', () => {
    const content = fs.readFileSync(relevantFiles.chatTypes, 'utf-8');
    
    // Should have JSDoc comments for ChannelType
    const channelTypeSection = content.substring(
      Math.max(0, content.indexOf('ChannelType') - 200),
      content.indexOf('ChannelType') + 100
    );
    
    expect(channelTypeSection).toMatch(/\/\*\*/);
  });

  /**
   * Consistency test: All three channel types should be handled consistently
   */
  test('Consistency: All channel types should follow the same naming pattern', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    // All channel type checks should use startsWith pattern
    const privateCheck = content.match(/channelName\.startsWith\(['"]private-['"]\)/);
    const presenceCheck = content.match(/channelName\.startsWith\(['"]presence-['"]\)/);
    
    expect(privateCheck).toBeTruthy();
    expect(presenceCheck).toBeTruthy();
  });

  /**
   * Property-Based Test: Channel type prefixes should be validated correctly
   * **Validates: Requirements 4.5**
   */
  test('Property 8.4: Channel type prefixes are validated correctly', () => {
    const content = fs.readFileSync(relevantFiles.authEndpoint, 'utf-8');
    
    fc.assert(
      fc.property(
        fc.constantFrom('private', 'presence'),
        (prefix) => {
          // Check that the prefix is used with startsWith
          const pattern = new RegExp(`startsWith\\(['"]${prefix}-['"]\\)`);
          return pattern.test(content);
        }
      ),
      { numRuns: 100 }
    );
  });
});
