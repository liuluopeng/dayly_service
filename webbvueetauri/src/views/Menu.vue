<script setup lang="ts">
import { reactive, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { getMenuItems } from '../config/menu';

const { t } = useI18n();
const menuItems = computed(() => getMenuItems(t));
const expanded = reactive<Record<string, boolean>>({});

function toggleExpand(path: string) {
  expanded[path] = !expanded[path];
}
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <header class="bg-white shadow-sm">
      <div class="container mx-auto px-4 py-3 flex justify-between items-center">
        <h1 class="text-xl font-bold text-blue-600">{{ t('menu.title') }}</h1>
      </div>
    </header>

    <div class="container mx-auto px-4 py-8">
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h2 class="text-xl font-semibold mb-6">{{ t('menu.title') }}</h2>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- 菜单项 -->
          <template v-for="item in menuItems" :key="item.path">
            <!-- 有子菜单的项 -->
            <div v-if="item.children">
              <button @click="toggleExpand(item.path)"
                class="w-full p-4 bg-blue-50 border border-blue-200 rounded-lg hover:bg-blue-100 transition-colors text-left cursor-pointer">
                <div class="flex items-center">
                  <div class="w-12 h-12 bg-blue-500 text-white rounded-full flex items-center justify-center mr-4">
                    <span class="text-xl font-bold">{{ item.icon }}</span>
                  </div>
                  <div class="flex-1">
                    <h3 class="text-lg font-medium text-blue-700">{{ item.title }}</h3>
                    <p class="text-gray-600">{{ item.description }}</p>
                  </div>
                  <svg
                    class="w-5 h-5 text-blue-400 transition-transform duration-200 flex-shrink-0"
                    :class="{ 'rotate-90': expanded[item.path] }"
                    fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </div>
              </button>

              <!-- 子菜单 -->
              <div
                v-show="expanded[item.path]"
                class="mt-2 ml-16 space-y-2">
                <router-link v-for="child in item.children" :key="child.path" :to="child.path"
                  class="block p-2 bg-white border border-blue-100 rounded-md hover:bg-blue-50 transition-colors">
                  <div class="flex items-center">
                    <span class="mr-2">{{ child.icon }}</span>
                    <div>
                      <h4 class="font-medium text-blue-600">{{ child.title }}</h4>
                      <p class="text-sm text-gray-500">{{ child.description }}</p>
                    </div>
                  </div>
                </router-link>
              </div>
            </div>
            <!-- 没有子菜单的项 -->
            <router-link v-else :to="item.path"
              class="block p-4 bg-blue-50 border border-blue-200 rounded-lg hover:bg-blue-100 transition-colors">
              <div class="flex items-center">
                <div class="w-12 h-12 bg-blue-500 text-white rounded-full flex items-center justify-center mr-4">
                  <span class="text-xl font-bold">{{ item.icon }}</span>
                </div>
                <div class="flex-1">
                  <h3 class="text-lg font-medium text-blue-700">{{ item.title }}</h3>
                  <p class="text-gray-600">{{ item.description }}</p>
                </div>
              </div>
            </router-link>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>