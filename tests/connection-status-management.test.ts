/**
 * Property-Based Tests for Connection Status Management
 * Feature: documentation-and-deployment-enhancements
 * Property 9: Bağlantı Durumu Yönetimi
 * 
 * **Validates: Requirements 4.9**
 * 
 * Tests that for every Pusher connection state change (connected, disconnected, reconnecting),
 * the application includes appropriate event handlers.
 */

import * as fc from 'fast-check';
import * as fs from 'fs';
import * as path from 'path';

describe('Property 9: Connection Status Management', () => {
  /**
   * Define all possible Pusher connection states
   * Based on Pusher.js documentation and our ConnectionStatus type
   */
  const CONNECTION_STATES = [
    'connected',
    'connecting',
    'disconnected',
    'unavailable',
    'failed'
  ] as const;

  /**
   * Helper function to read file content
   */
  function readFileContent(filePath: string): string {
    const fullPath = path.join(process.cwd(), filePath);
    if (!fs.existsSync(fullPath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    return fs.readFileSync(fullPath, 'utf-8');
  }

  /**
   * Helper function to check if a file contains an event handler for a specific state
   * Looks for patterns like:
   * - pusherClient.connection.bind('connected', ...)
   * - connection.bind('connected', ...)
   * - .bind('connected', handler)
   */
  function hasEventHandler(content: string, state: string): boolean {
    // Pattern 1: pusherClient.connection.bind('state', ...)
    const pattern1 = new RegExp(`pusherClient\\.connection\\.bind\\s*\\(\\s*['"\`]${state}['"\`]`, 'i');
    
    // Pattern 2: connection.bind('state', ...)
    const pattern2 = new RegExp(`connection\\.bind\\s*\\(\\s*['"\`]${state}['"\`]`, 'i');
    
    // Pattern 3: .bind('state', ...)
    const pattern3 = new RegExp(`\\.bind\\s*\\(\\s*['"\`]${state}['"\`]`, 'i');
    
    return pattern1.test(content) || pattern2.test(content) || pattern3.test(content);
  }

  /**
   * Helper function to check if a file contains automatic reconnection logic
   * Looks for patterns like:
   * - pusherClient.connect()
   * - connection.connect()
   * - reconnect logic with setTimeout/setInterval
   */
  function hasReconnectionLogic(content: string): boolean {
    // Pattern 1: pusherClient.connect()
    const pattern1 = /pusherClient\.connect\s*\(\s*\)/i;
    
    // Pattern 2: connection.connect()
    const pattern2 = /connection\.connect\s*\(\s*\)/i;
    
    // Pattern 3: setTimeout with reconnect logic
    const pattern3 = /setTimeout\s*\(\s*\(\s*\)\s*=>\s*\{[^}]*connect/i;
    
    // Pattern 4: Reconnection attempts tracking
    const pattern4 = /reconnect.*attempt/i;
    
    return pattern1.test(content) || pattern2.test(content) || 
           pattern3.test(content) || pattern4.test(content);
  }

  /**
   * Helper function to check if a file contains status display logic
   * Looks for patterns that display connection status to users
   */
  function hasStatusDisplay(content: string): boolean {
    // Pattern 1: Status state variable
    const pattern1 = /useState.*status|status.*useState/i;
    
    // Pattern 2: Status display in JSX
    const pattern2 = /status.*connected|connected.*status/i;
    
    // Pattern 3: Visual indicators for status
    const pattern3 = /bg-green|bg-red|bg-yellow|bg-gray.*status|status.*bg-green|bg-red|bg-yellow|bg-gray/i;
    
    // Pattern 4: Status text display
    const pattern4 = /Connected|Disconnected|Connecting|Unavailable|Failed/;
    
    return pattern1.test(content) || pattern2.test(content) || 
           pattern3.test(content) || pattern4.test(content);
  }

  /**
   * Helper function to extract all event handlers from content
   */
  function extractEventHandlers(content: string): string[] {
    const handlers: string[] = [];
    const bindPattern = /\.bind\s*\(\s*['"`]([^'"`]+)['"`]/g;
    
    let match;
    while ((match = bindPattern.exec(content)) !== null) {
      handlers.push(match[1]);
    }
    
    return handlers;
  }

  /**
   * Helper function to check if content has cleanup logic (unbind)
   */
  function hasCleanupLogic(content: string): boolean {
    // Pattern 1: unbind or unbind_all
    const pattern1 = /\.unbind\s*\(|\.unbind_all\s*\(/i;
    
    // Pattern 2: cleanup in useEffect return
    const pattern2 = /return\s*\(\s*\)\s*=>\s*\{[^}]*unbind/i;
    
    return pattern1.test(content) || pattern2.test(content);
  }

  // File paths to check
  const CONNECTION_STATUS_COMPONENT = 'next-chat-app/components/chat/ConnectionStatus.tsx';
  const CHAT_ROOM_COMPONENT = 'next-chat-app/components/chat/ChatRoom.tsx';
  const PUSHER_LIB = 'next-chat-app/lib/pusher.ts';

  /**
   * Unit test: ConnectionStatus component file should exist
   */
  test('ConnectionStatus component file should exist', () => {
    expect(fs.existsSync(path.join(process.cwd(), CONNECTION_STATUS_COMPONENT))).toBe(true);
  });

  /**
   * Unit test: ConnectionStatus component should import Pusher client
   */
  test('ConnectionStatus component should import Pusher client', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    expect(content).toMatch(/import.*pusherClient.*from.*pusher/i);
  });

  /**
   * Unit test: ConnectionStatus type should be defined
   */
  test('ConnectionStatus type should be defined in types', () => {
    const typesContent = readFileContent('next-chat-app/types/chat.ts');
    expect(typesContent).toMatch(/type\s+ConnectionStatus/i);
    
    // Check that all required states are in the type definition
    CONNECTION_STATES.forEach(state => {
      expect(typesContent).toMatch(new RegExp(`['"\`]${state}['"\`]`));
    });
  });

  /**
   * Property-Based Test: For every connection state, there must be an event handler
   * **Validates: Requirements 4.9**
   */
  test('Property 9.1: For every connection state, there must be an event handler', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    fc.assert(
      fc.property(
        fc.constantFrom(...CONNECTION_STATES),
        (state) => {
          const result = hasEventHandler(content, state);
          
          if (!result) {
            console.log(`\nMissing event handler for connection state: ${state}`);
            console.log(`  Expected pattern: connection.bind('${state}', handler)`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: For every connection state handler, there must be a corresponding unbind in cleanup
   * **Validates: Requirements 4.9**
   */
  test('Property 9.2: For every connection state handler, there must be cleanup logic', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    const handlers = extractEventHandlers(content);
    
    // First check that cleanup logic exists
    expect(hasCleanupLogic(content)).toBe(true);
    
    fc.assert(
      fc.property(
        fc.constantFrom(...CONNECTION_STATES),
        (state) => {
          // If there's a bind for this state, there should be an unbind
          if (hasEventHandler(content, state)) {
            const unbindPattern = new RegExp(`\\.unbind\\s*\\(\\s*['"\`]${state}['"\`]`, 'i');
            const result = unbindPattern.test(content);
            
            if (!result) {
              console.log(`\nMissing unbind for connection state: ${state}`);
              console.log(`  Expected pattern: connection.unbind('${state}', handler)`);
            }
            
            return result;
          }
          return true; // If no bind, no unbind needed
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Connection status component must have automatic reconnection logic
   * **Validates: Requirements 4.9**
   */
  test('Property 9.3: Connection status component must have automatic reconnection logic', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    expect(hasReconnectionLogic(content)).toBe(true);
    
    // Additional checks for reconnection logic
    expect(content).toMatch(/reconnect/i);
    expect(content).toMatch(/connect\s*\(\s*\)/);
  });

  /**
   * Property-Based Test: Connection status must be displayed to users
   * **Validates: Requirements 4.9**
   */
  test('Property 9.4: Connection status must be displayed to users', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    expect(hasStatusDisplay(content)).toBe(true);
    
    // Check that all connection states have visual representation
    fc.assert(
      fc.property(
        fc.constantFrom(...CONNECTION_STATES),
        (state) => {
          // Check if the state is mentioned in the component (for display purposes)
          const statePattern = new RegExp(state, 'i');
          const result = statePattern.test(content);
          
          if (!result) {
            console.log(`\nConnection state not displayed: ${state}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Each connection state should have a visual indicator
   * **Validates: Requirements 4.9**
   */
  test('Property 9.5: Each connection state should have a visual indicator', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Define expected visual indicators for each state
    const stateIndicators: Record<string, string[]> = {
      'connected': ['green', 'success'],
      'connecting': ['yellow', 'warning', 'pulse'],
      'disconnected': ['gray', 'inactive'],
      'unavailable': ['orange', 'warning'],
      'failed': ['red', 'error', 'danger']
    };
    
    fc.assert(
      fc.property(
        fc.constantFrom(...CONNECTION_STATES),
        (state) => {
          const indicators = stateIndicators[state];
          
          // Check if at least one indicator is present for this state
          const hasIndicator = indicators.some(indicator => {
            const pattern = new RegExp(`${state}[^}]*${indicator}|${indicator}[^}]*${state}`, 'i');
            return pattern.test(content);
          });
          
          if (!hasIndicator) {
            console.log(`\nMissing visual indicator for state: ${state}`);
            console.log(`  Expected one of: ${indicators.join(', ')}`);
          }
          
          return hasIndicator;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Property-Based Test: Reconnection should use exponential backoff
   * **Validates: Requirements 4.9**
   */
  test('Property 9.6: Reconnection should use exponential backoff strategy', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for exponential backoff patterns
    const hasExponentialBackoff = 
      /Math\.pow\s*\(\s*2\s*,/i.test(content) || // Math.pow(2, attempts)
      /\*\*\s*reconnect/i.test(content) || // 2 ** attempts
      /exponential/i.test(content); // Comment mentioning exponential
    
    expect(hasExponentialBackoff).toBe(true);
    
    // Check for max attempts limit
    const hasMaxAttempts = /maxAttempts|max.*attempts/i.test(content);
    expect(hasMaxAttempts).toBe(true);
  });

  /**
   * Property-Based Test: Connection state changes should trigger callbacks
   * **Validates: Requirements 4.9**
   */
  test('Property 9.7: Connection state changes should trigger optional callbacks', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for callback prop
    const hasCallbackProp = /onStatusChange|onChange|callback/i.test(content);
    expect(hasCallbackProp).toBe(true);
    
    fc.assert(
      fc.property(
        fc.constantFrom(...CONNECTION_STATES),
        (state) => {
          // Check if callback is invoked for this state
          const callbackPattern = new RegExp(`${state}[^}]*onStatusChange|onStatusChange[^}]*${state}`, 'i');
          const result = callbackPattern.test(content);
          
          if (!result) {
            console.log(`\nCallback not invoked for state: ${state}`);
          }
          
          return result;
        }
      ),
      { numRuns: 100 }
    );
  });

  /**
   * Edge case test: Component should handle initial connection state
   */
  test('Edge case: Component should set initial connection state', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for initial state setting
    expect(content).toMatch(/useState.*connecting|initialState|connection\.state/i);
  });

  /**
   * Edge case test: Component should handle rapid state changes
   */
  test('Edge case: Component should handle cleanup on unmount', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for useEffect cleanup
    expect(content).toMatch(/return\s*\(\s*\)\s*=>\s*\{/);
    expect(hasCleanupLogic(content)).toBe(true);
  });

  /**
   * Edge case test: Manual reconnect should be available for failed connections
   */
  test('Edge case: Manual reconnect should be available for failed connections', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for manual reconnect button or function
    expect(content).toMatch(/manual.*reconnect|reconnect.*button|handleManualReconnect/i);
  });

  /**
   * Edge case test: Component should show reconnection attempts
   */
  test('Edge case: Component should display reconnection attempts', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for reconnection attempts display
    expect(content).toMatch(/reconnectAttempts|attempt.*\d+\/\d+/i);
  });

  /**
   * Integration test: Verify all connection states are handled
   */
  test('Integration test: All connection states should have complete handling', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    const missingHandlers: string[] = [];
    const handledStates: string[] = [];
    
    CONNECTION_STATES.forEach(state => {
      if (hasEventHandler(content, state)) {
        handledStates.push(state);
      } else {
        missingHandlers.push(state);
      }
    });
    
    if (missingHandlers.length > 0) {
      console.log('\nMissing event handlers for states:');
      missingHandlers.forEach(state => console.log(`  - ${state}`));
    }
    
    console.log(`\nConnection state coverage: ${handledStates.length}/${CONNECTION_STATES.length} states handled`);
    
    // All states should have handlers
    expect(missingHandlers.length).toBe(0);
  });

  /**
   * Integration test: Verify connection status is used in ChatRoom
   */
  test('Integration test: ChatRoom should use connection status', () => {
    const content = readFileContent(CHAT_ROOM_COMPONENT);
    
    // Check if ChatRoom monitors connection state
    const hasConnectionMonitoring = 
      /connection.*state|isConnected|connectionStatus/i.test(content);
    
    expect(hasConnectionMonitoring).toBe(true);
  });

  /**
   * Integration test: Verify Pusher client is properly configured
   */
  test('Integration test: Pusher client should be properly configured', () => {
    const content = readFileContent(PUSHER_LIB);
    
    // Check for Pusher client initialization
    expect(content).toMatch(/new Pusher/i);
    
    // Check for connection configuration
    expect(content).toMatch(/wsHost|wsPort|forceTLS|encrypted/i);
  });

  /**
   * Unit test: Verify error event handler exists
   */
  test('Unit test: Error event handler should exist', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for error event handler
    expect(hasEventHandler(content, 'error')).toBe(true);
  });

  /**
   * Unit test: Verify connection status component exports correctly
   */
  test('Unit test: ConnectionStatus component should be exported', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for export
    expect(content).toMatch(/export\s+default\s+function\s+ConnectionStatus|export\s+default\s+ConnectionStatus/i);
  });

  /**
   * Property-Based Test: Connection status component should be accessible
   * **Validates: Requirements 4.9**
   */
  test('Property 9.8: Connection status should have accessibility attributes', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for accessibility attributes
    const hasAriaLabel = /aria-label/i.test(content);
    const hasTitle = /title=/i.test(content);
    
    expect(hasAriaLabel || hasTitle).toBe(true);
  });

  /**
   * Property-Based Test: Connection status should be customizable
   * **Validates: Requirements 4.9**
   */
  test('Property 9.9: Connection status component should accept props for customization', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for props interface or type
    expect(content).toMatch(/interface.*Props|type.*Props/i);
    
    // Check for common customization props
    const hasCustomizationProps = 
      /showText|className|onStatusChange/i.test(content);
    
    expect(hasCustomizationProps).toBe(true);
  });

  /**
   * Performance test: Reconnection should not create infinite loops
   */
  test('Performance: Reconnection should have maximum attempts limit', () => {
    const content = readFileContent(CONNECTION_STATUS_COMPONENT);
    
    // Check for max attempts check
    expect(content).toMatch(/reconnectAttempts\s*<\s*maxAttempts|attempts.*<.*max/i);
    
    // Check that reconnection stops after max attempts
    expect(content).toMatch(/max.*attempts.*reached|reached.*max.*attempts/i);
  });
});
