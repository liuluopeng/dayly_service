<script setup lang="ts">
import { SUB_VALUE } from '../../../config/key';
import { reactive, watch, computed } from 'vue';

import type { KEY_PERMUTATION_VALUE } from '../../../config/key';

type Props = {
  unit: number;
  backgroundColor: string;
  color: string;
  value: string;
  code: string;
  keysPressed: any;
  heightType: string;
  type: string;
  area: Array<Array<KEY_PERMUTATION_VALUE>> | null;
  currentSystem: string;
  isActiveKey: boolean;
};

const props = withDefaults(defineProps<Partial<Props>>(), {
  unit: 1,
  backgroundColor: '#4F5767',
  color: '#fff',
  value: '',
  code: '',
  keysPressed: {},
  heightType: 'normal',
  type: 'normal',
  area: null,
  currentSystem: 'win',
  isActiveKey: true
});

const state = reactive({
  isActive: false // 是否已经被点击过了
});

watch(
  () => props.keysPressed,
  (val) => {
    if (val) {
      const triggeredKeys = Object.keys(val);
      if (triggeredKeys.includes(props.code)) {
        state.isActive = true;
      }
    }
  },
  {
    deep: true
  }
);

const isKeyPressed = computed(() => {
  return props.keysPressed[props.code];
});

const subValue = computed(() => {
  return SUB_VALUE[props.value];
});
</script>
<template>
  <div class="y-single-key__inner flex flex-col" v-if="type === 'inner' && area">
    <div class="y-single-key__inner-item flex items-center justify-between" v-for="(v, i) in area" :key="i">
      <single-key v-for="item in v" :key="item.code" height-type="half"
        :code="currentSystem === 'mac' && item.macCode ? item.macCode : item.code"
        :value="currentSystem === 'mac' && item.macValue ? item.macValue : item.value" :unit="item.unit"
        :keys-pressed="keysPressed"></single-key>
    </div>
  </div>
  <div v-else class="y-single-key flex items-center w-[2.2rem] h-[2.2rem] text-[1.8rem] font-bold rounded-[0.5rem] shadow-[0.2rem_0.2rem_0.4rem_rgba(0,0,0,0.4)] p-[0.6rem] my-[0.4rem] mx-0 relative" :class="[
    code ? '' : 'bg-transparent! shadow-none!',
    unit === 1 ? 'w-[2.2rem]' : unit === 1.25 ? 'w-[2.75rem]' : unit === 1.5 ? 'w-[3.3rem]' : unit === 1.75 ? 'w-[3.85rem]' : unit === 2 ? 'w-[4.4rem]' : unit === 2.25 ? 'w-[4.95rem]' : unit === 2.5 ? 'w-[5.5rem]' : unit === 2.75 ? 'w-[6.05rem]' : unit === 3 ? 'w-[6.6rem]' : unit === 6 ? 'w-[13.2rem]' : unit === 7 ? 'w-[15.4rem]' : 'w-[2.2rem]',
    subValue ? 'text-[1.2rem] flex-col items-start justify-between' : '',
    value.length > 1 ? 'text-[1.2rem]' : 'items-start leading-[1rem]',
    state.isActive ? 'y-single-key--active' : '',
    isKeyPressed ? 'y-single-key--pressed' : '',
    heightType === 'half' ? 'h-[1.1rem] my-[0.1rem] mx-0' : '',
    !isActiveKey ? 'opacity-30 brightness-50 transition-opacity duration-300 transition-[filter] duration-300' : ''
  ]" :style="{
    backgroundColor,
    color
  }">
    <span v-if="subValue" class="y-single-key__sub-value">{{ subValue }}</span>
    <span class="y-single-key__value">{{ value }}</span>
  </div>
</template>
<style>
.y-single-key::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border-radius: 0.5rem;
  background-color: rgba(255, 255, 255, 0);
  transition: background-color 0.1s;
}

.y-single-key--active::after {
  background-color: rgba(255, 255, 255, 0.2);
}

.y-single-key--pressed::after {
  background-color: rgba(255, 255, 255, 0.6);
}

.bg-transparent\!.y-single-key::after {
  content: none;
}
</style>
