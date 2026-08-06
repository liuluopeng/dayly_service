<script setup lang="ts">
import { ref } from 'vue';

const display = ref('0');
const expression = ref('');

function append(ch: string) {
  if (display.value === '0' && ch !== '.') display.value = ch;
  else display.value += ch;
}

function clear() {
  display.value = '0';
  expression.value = '';
}

function backspace() {
  if (display.value.length > 1) display.value = display.value.slice(0, -1);
  else display.value = '0';
}

function compute(expr: string): string {
  // 仅允许数字、四则运算符、括号、小数点（受限求值，防止注入）
  if (!/^[\d+\-*/().\s%]+$/.test(expr)) throw new Error('非法表达式');
  const value = Function(`"use strict"; return (${expr})`)();
  if (typeof value !== 'number' || !isFinite(value)) throw new Error('无效结果');
  return String(Math.round(value * 1e10) / 1e10);
}

function press(op: string) {
  const last = display.value.slice(-1);
  if ('+-*/%'.includes(op) && '+-*/%'.includes(last)) {
    display.value = display.value.slice(0, -1) + op;
    return;
  }
  display.value += op;
}

function evaluate() {
  try {
    expression.value = `${display.value} =`;
    display.value = compute(display.value);
  } catch (e) {
    expression.value = '错误';
    display.value = (e as Error).message;
  }
}

const buttons = [
  'C', '⌫', '%', '/',
  '7', '8', '9', '*',
  '4', '5', '6', '-',
  '1', '2', '3', '+',
  '0', '.', '=', ''
];

function onButton(b: string) {
  if (b === 'C') clear();
  else if (b === '⌫') backspace();
  else if (b === '=') evaluate();
  else if ('+-*/%'.includes(b)) press(b);
  else append(b);
}
</script>

<template>
  <div class="p-4 max-w-[420px] mx-auto">
    <h2 class="mb-4 text-[1.4em]">计算器</h2>

    <div class="bg-white rounded-xl shadow p-4">
      <div class="text-right mb-3 px-2">
        <div class="text-[#999] text-sm h-6 overflow-hidden text-ellipsis whitespace-nowrap">{{ expression }}</div>
        <div class="text-[2.2em] font-light overflow-hidden text-ellipsis whitespace-nowrap">{{ display }}</div>
      </div>

      <div class="grid grid-cols-4 gap-2">
        <template v-for="b in buttons" :key="b">
          <button
            v-if="b"
            class="h-14 rounded-lg text-lg cursor-pointer border-none transition-colors duration-150 disabled:opacity-40"
            :class="{
              'bg-[#1976d2] text-white hover:bg-[#1565c0]': b === '=',
              'bg-[#f0f0f0] hover:bg-[#e0e0e0]': b !== '=' && !'+-*/%'.includes(b) && b !== 'C' && b !== '⌫',
              'bg-[#e8f4fd] text-[#2980b9] hover:bg-[#d0e8f7]': '+-*/%'.includes(b),
              'bg-[#ffebee] text-[#c62828] hover:bg-[#ffcdd2]': b === 'C' || b === '⌫',
            }"
            @click="onButton(b)"
          >{{ b }}</button>
          <div v-else class="h-14"></div>
        </template>
      </div>
    </div>
  </div>
</template>
