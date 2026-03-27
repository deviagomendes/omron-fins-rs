#!/usr/bin/env node

/**
 * omron-fins Binary Test Script
 * 
 * Tests the @omron-fins/native binary on Linux.
 * Run with: node scripts/test-binary.js
 * 
 * Requirements:
 *   - Node.js >= 18
 *   - @omron-fins/native installed (npm install)
 *   - A running Omron PLC at 192.168.1.250 (or modify PLC_HOST)
 */

const { FinsClient, FinsMemoryArea, FinsDataType, DEFAULT_FINS_PORT, MAX_WORDS_PER_COMMAND } = require('@omron-fins/native');

const PLC_HOST = process.env.PLC_HOST || '192.168.1.250';
const PLC_SOURCE_NODE = parseInt(process.env.PLC_SOURCE_NODE || '1', 10);
const PLC_DEST_NODE = parseInt(process.env.PLC_DEST_NODE || '0', 10);

console.log('='.repeat(60));
console.log('Omron FINS Binary Test');
console.log('='.repeat(60));
console.log(`Platform: ${process.platform} ${process.arch}`);
console.log(`Node.js: ${process.version}`);
console.log(`PLC Host: ${PLC_HOST}`);
console.log(`Source Node: ${PLC_SOURCE_NODE}, Dest Node: ${PLC_DEST_NODE}`);
console.log('='.repeat(60));

async function testBasicOperations() {
  console.log('\n--- Test: Basic Operations ---');
  
  const client = new FinsClient(PLC_HOST, PLC_SOURCE_NODE, PLC_DEST_NODE, {
    timeoutMs: 3000,
    port: DEFAULT_FINS_PORT
  });

  console.log('✓ FinsClient created');

  // Test read
  const words = await client.read('DM', 0, 5);
  console.log(`✓ Read 5 words from DM0: [${words.join(', ')}]`);

  // Test read with enum
  const words2 = await client.read(FinsMemoryArea.DM, 0, 5);
  console.log(`✓ Read with enum: [${words2.join(', ')}]`);

  // Test bit read (if bit area)
  try {
    const bit = await client.readBit('CIO', 0, 0);
    console.log(`✓ Read bit CIO 0.00: ${bit}`);
  } catch (err) {
    console.log(`⚠ Bit read skipped (may not be available): ${err.message}`);
  }

  // Test read multiple
  const multiRead = await client.readMultiple([
    { area: 'DM', address: 0, bit: null },
    { area: 'DM', address: 10, bit: null },
    { area: 'CIO', address: 0, bit: 0 }
  ]);
  console.log(`✓ Multi-read: [${multiRead.join(', ')}]`);

  console.log('\n--- Test: Typed Values ---');

  // Test f32 read
  try {
    const f32Val = await client.readF32('DM', 100);
    console.log(`✓ Read f32 from DM100: ${f32Val}`);
  } catch (err) {
    console.log(`⚠ f32 read skipped: ${err.message}`);
  }

  // Test i32 read
  try {
    const i32Val = await client.readI32('DM', 102);
    console.log(`✓ Read i32 from DM102: ${i32Val}`);
  } catch (err) {
    console.log(`⚠ i32 read skipped: ${err.message}`);
  }

  // Test string read
  try {
    const str = await client.readString('DM', 200, 5);
    console.log(`✓ Read string from DM200: "${str}"`);
  } catch (err) {
    console.log(`⚠ String read skipped: ${err.message}`);
  }

  console.log('\n--- Test: Write Operations ---');

  // Test write
  await client.write('DM', 1000, [0x1234, 0x5678, 0xABCD]);
  console.log('✓ Wrote 3 words to DM1000');

  // Test write with enum
  await client.write(FinsMemoryArea.DM, 1003, [0x0001, 0x0002]);
  console.log('✓ Wrote with enum to DM1003');

  // Test fill
  await client.fill('DM', 1100, 10, 0x0000);
  console.log('✓ Filled DM1100-DM1109 with 0x0000');

  // Test transfer
  await client.transfer('DM', 1000, 'DM', 1200, 3);
  console.log('✓ Transferred DM1000-DM1002 to DM1200-DM1202');

  // Test bit write
  try {
    await client.writeBit('WR', 0, 0, true);
    console.log('✓ Wrote bit WR 0.00 = true');
  } catch (err) {
    console.log(`⚠ Bit write skipped: ${err.message}`);
  }

  console.log('\n--- Test: Struct Operations ---');

  // Test struct read
  try {
    const struct = await client.readStruct('DM', 1300, ['UINT', 'INT', 'REAL']);
    console.log(`✓ Read struct from DM1300:`, JSON.stringify(struct));
  } catch (err) {
    console.log(`⚠ Struct read skipped: ${err.message}`);
  }

  // Test struct write
  try {
    await client.writeStruct('DM', 1400, [
      { type: 'UINT', value: '123' },
      { type: 'INT', value: '-456' },
      { type: 'REAL', value: '3.14159' }
    ]);
    console.log('✓ Wrote struct to DM1400');
  } catch (err) {
    console.log(`⚠ Struct write skipped: ${err.message}`);
  }

  console.log('\n--- Test: Utility Functions ---');

  // Test utility functions (these are sync)
  const { getBit, setBit, toggleBit, wordToBits, bitsToWord, getOnBits, countOnBits, formatBinary, formatHex } = require('@omron-fins/native');

  const testWord = 0b1010010111000011;

  console.log(`✓ getBit(0x${testWord.toString(16)}, 0): ${getBit(testWord, 0)}`);
  console.log(`✓ setBit: 0x${setBit(testWord, 4, true).toString(16)}`);
  console.log(`✓ toggleBit: 0x${toggleBit(testWord, 0).toString(16)}`);
  console.log(`✓ wordToBits: [${wordToBits(testWord).slice(0, 8).join(', ')}, ...]`);
  console.log(`✓ bitsToWord: ${bitsToWord([true, false, true, false, true, false, true, false, false, false, false, false, false, false, false, false])}`);
  console.log(`✓ getOnBits: [${getOnBits(testWord).join(', ')}]`);
  console.log(`✓ countOnBits: ${countOnBits(testWord)}`);
  console.log(`✓ formatBinary: ${formatBinary(testWord)}`);
  console.log(`✓ formatHex: ${formatHex(testWord)}`);

  console.log('\n--- Test: Constants ---');
  console.log(`DEFAULT_FINS_PORT: ${DEFAULT_FINS_PORT}`);
  console.log(`MAX_WORDS_PER_COMMAND: ${MAX_WORDS_PER_COMMAND}`);

  console.log('\n' + '='.repeat(60));
  console.log('All tests passed!');
  console.log('='.repeat(60));
}

async function main() {
  try {
    await testBasicOperations();
  } catch (error) {
    console.error('\n✗ Test failed:', error.message);
    console.error('Stack:', error.stack);
    process.exit(1);
  }
}

main();
