<!-- skills-manager/src/components/SkillCard.vue -->
<template>
  <t-card
    :class="['skill-card', { selected: selected, 'is-selected-skill': skill.is_selected }]"
    :bordered="true"
    :hover-shadow="true"
    role="button"
    :aria-selected="selected"
    tabindex="0"
    @click="onClick"
    @keydown.enter="onClick"
    @keydown.space.prevent="onClick"
  >
    <!-- Selection badge -->
    <div v-if="skill.is_selected" class="selection-badge">
      <t-tag theme="primary" size="small" variant="light">已选</t-tag>
    </div>

    <template #header>
      <div class="card-header">
        <span class="skill-icon">◈</span>
        <span class="skill-name">{{ skill.name }}</span>
      </div>
    </template>

    <div class="skill-description">
      {{ skill.description || '暂无描述' }}
    </div>

    <template #footer>
      <div class="skill-meta">
        <t-tag size="small" theme="primary">{{ repoName }}</t-tag>
      </div>
    </template>
  </t-card>
</template>

<script setup lang="ts">
import type { SkillMeta } from '@/stores/skills'

const props = defineProps<{
  skill: SkillMeta
  repoName: string
  selected: boolean
}>()

const emit = defineEmits<{
  (e: 'click', skill: SkillMeta): void
}>()

function onClick() {
  emit('click', props.skill)
}
</script>

<style scoped>
.skill-card {
  position: relative;
  background: #fff;
  border: 1px solid #e7e7e7;
  border-radius: 6px;
  cursor: pointer;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
}

.skill-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.skill-card.selected {
  border-color: var(--td-brand-color);
  box-shadow: 0 0 0 2px var(--td-brand-color-focus);
}

.skill-card.is-selected-skill {
  border-color: var(--td-brand-color);
}

.selection-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 1;
}

.skill-card:focus-visible {
  outline: 2px solid var(--td-brand-color);
  outline-offset: 2px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skill-icon {
  color: var(--td-brand-color);
}

.skill-name {
  font-weight: 600;
}

.skill-description {
  font-size: 14px;
  color: var(--td-text-color-secondary);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.skill-meta {
  display: flex;
  align-items: center;
}
</style>