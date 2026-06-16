<script setup lang="ts">
import { getMenuItems } from './config/menu';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useApiReinit } from './composables/useApiReinit';
import { useAuth } from './composables/useAuth';

const { t } = useI18n();
const { reinitApi } = useApiReinit();
const { username, isLoggedIn } = useAuth();
const menuItems = computed(() => getMenuItems(t));
const sidebarPinned = ref(true);
const isHovering = ref(false);
const isFullscreen = ref(false);

const sidebarExpanded = computed(() => sidebarPinned.value || isHovering.value);

function togglePin() {
  sidebarPinned.value = !sidebarPinned.value;
}

function onFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
}

onMounted(() => {
  reinitApi();
  document.addEventListener('fullscreenchange', onFullscreenChange);
});

onUnmounted(() => {
  document.removeEventListener('fullscreenchange', onFullscreenChange);
});
</script>

<template>
  <div class="min-h-screen bg-gray-100 text-gray-900 font-sans flex">
    <!-- Sidebar Navigation -->
    <nav v-show="!isFullscreen"
      @mouseenter="isHovering = true"
      @mouseleave="isHovering = false"
      :class="[
        'bg-white shadow-md h-screen fixed left-0 top-0 z-10 overflow-y-auto overflow-x-hidden transition-all duration-300',
        sidebarExpanded ? 'w-64' : 'w-16'
      ]">
      <div class="p-4 border-b">
        <div class="flex items-center justify-between">
          <h1 v-show="sidebarExpanded" class="text-xl font-bold text-blue-600">App Menu</h1>
          <button @click="togglePin"
          class="p-1 rounded-lg hover:bg-gray-100 transition-colors text-gray-500 hover:text-gray-700"
          :title="sidebarPinned ? t('app.sidebar.unpin') : t('app.sidebar.pin')">
          <svg v-if="sidebarPinned" class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
            <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
          </svg>
          <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" transform="rotate(45 12 12)"/>
          </svg>
        </button>
        </div>
        <div v-show="sidebarExpanded && isLoggedIn" class="mt-1 text-xs text-gray-500 truncate">
          {{ username }}
        </div>
      </div>
      <div class="py-4">
        <router-link to="/"
          :class="[
            'block px-4 py-3 text-gray-700 hover:bg-blue-50 hover:text-blue-600 transition-colors border-l-4 border-transparent hover:border-blue-500',
            sidebarExpanded ? '' : 'text-center'
          ]"
          :title="sidebarExpanded ? '' : 'Home'">
          <span v-if="sidebarExpanded">Home</span>
          <span v-else>H</span>
        </router-link>
        <router-link to="/menu"
          :class="[
            'block px-4 py-3 text-gray-700 hover:bg-blue-50 hover:text-blue-600 transition-colors border-l-4 border-transparent hover:border-blue-500',
            sidebarExpanded ? '' : 'text-center'
          ]"
          :title="sidebarExpanded ? '' : 'Menu'">
          <span v-if="sidebarExpanded">Menu</span>
          <span v-else>M</span>
        </router-link>

        <!-- 菜单项 -->
        <template v-for="item in menuItems" :key="item.path">
          <!-- 有子菜单的项 -->
          <div v-if="item.children">
            <div
              :class="[
                'px-4 py-3 text-gray-700 hover:bg-blue-50 hover:text-blue-600 transition-colors border-l-4 border-transparent hover:border-blue-500 cursor-pointer',
                sidebarExpanded ? '' : 'text-center'
              ]"
              :title="sidebarExpanded ? '' : item.title">
              <span v-if="sidebarExpanded">{{ item.title }}</span>
              <span v-else>{{ item.icon || item.title[0] }}</span>
            </div>
            <!-- 子菜单 -->
            <div v-show="sidebarExpanded" class="ml-4 border-l-2 border-gray-200">
              <router-link v-for="child in item.children" :key="child.path" :to="child.path"
                class="block px-4 py-2 text-sm text-gray-600 hover:bg-blue-50 hover:text-blue-600 transition-colors">
                {{ child.title }}
              </router-link>
            </div>
          </div>
          <!-- 没有子菜单的项 -->
          <router-link v-else :to="item.path"
            :class="[
              'block px-4 py-3 text-gray-700 hover:bg-blue-50 hover:text-blue-600 transition-colors border-l-4 border-transparent hover:border-blue-500',
              sidebarExpanded ? '' : 'text-center'
            ]"
            :title="sidebarExpanded ? '' : item.title">
            <span v-if="!sidebarExpanded">{{ item.icon || item.title[0] }}</span>
            <span v-else>{{ item.title }}</span>
          </router-link>
        </template>
      </div>
    </nav>

    <!-- Main Content -->
    <div :class="[
      'flex-1 min-w-0 overflow-x-hidden transition-all duration-300',
      isFullscreen ? 'ml-0' : (sidebarExpanded ? 'ml-64' : 'ml-16')
    ]">
      <!-- Router View -->
      <router-view v-slot="{ Component }">
        <component :is="Component" />
      </router-view>
    </div>
  </div>
</template>

<style scoped>
/* No animations */
</style>
