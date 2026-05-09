<template>
  <div class="home-content">
    <!-- Skills grid -->
    <div v-if="filteredSkills.length" class="skill-grid">
      <SkillCard
        v-for="skill in filteredSkills"
        :key="skill.path"
        :skill="skill"
        :repo-name="getRepoName(skill.repo_id)"
        :selected="skillsStore.selectedSkill?.path === skill.path"
        @click="onSkillClick"
      />
    </div>

    <!-- Empty state -->
    <div v-else class="empty-state">
      <t-empty description="暂无技能">
        <template #action>
          <t-button
            v-if="!configStore.config?.repositories?.length"
            theme="primary"
            @click="emit('addRepo')"
          >
            添加第一个仓库
          </t-button>
        </template>
      </t-empty>
    </div>

    <!-- Skill detail panel -->
    <SkillDetailPanel
      :skill="skillsStore.selectedSkill"
      :repo-name="selectedRepoName"
      @close="onPanelClose"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import SkillCard from '@/components/SkillCard.vue'
import SkillDetailPanel from '@/components/SkillDetailPanel.vue'
import type { SkillMeta } from '@/stores/skills'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()

const emit = defineEmits<{ (e: 'addRepo'): void }>()

const filteredSkills = computed(() => {
  let skills = skillsStore.skills
  if (skillsStore.currentRepoId) {
    skills = skills.filter(s => s.repo_id === skillsStore.currentRepoId)
  }
  if (skillsStore.searchQuery) {
    const query = skillsStore.searchQuery.toLowerCase()
    skills = skills.filter(
      s =>
        s.name.toLowerCase().includes(query) ||
        s.description.toLowerCase().includes(query)
    )
  }
  return skills
})

const selectedRepoName = computed(() => {
  if (!skillsStore.selectedSkill) return ''
  return getRepoName(skillsStore.selectedSkill.repo_id)
})

function getRepoName(repoId: string): string {
  return (
    configStore.config?.repositories.find(r => r.id === repoId)?.name ||
    'Unknown'
  )
}

function onSkillClick(skill: SkillMeta) {
  skillsStore.selectSkill(skill)
}

function onPanelClose() {
  skillsStore.selectSkill(null)
}
</script>

<style scoped>
.home-content {
  padding: 24px;
  min-height: 100%;
}

.skill-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
</style>