import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface SkillMeta {
  name: string
  description: string
  path: string
  repo_id: string
}

export const useSkillsStore = defineStore('skills', () => {
  const skills = ref<SkillMeta[]>([])
  const currentRepoId = ref<string | null>(null)
  const selectedSkill = ref<SkillMeta | null>(null)
  const searchQuery = ref('')

  function setSkills(skillsList: SkillMeta[]) {
    skills.value = skillsList
  }

  function setCurrentRepo(repoId: string | null) {
    currentRepoId.value = repoId
    selectedSkill.value = null
  }

  function selectSkill(skill: SkillMeta | null) {
    selectedSkill.value = skill
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function filteredSkills() {
    if (!searchQuery.value) return skills.value
    const query = searchQuery.value.toLowerCase()
    return skills.value.filter(skill =>
      skill.name.toLowerCase().includes(query) ||
      skill.description.toLowerCase().includes(query)
    )
  }

  return {
    skills,
    currentRepoId,
    selectedSkill,
    searchQuery,
    setSkills,
    setCurrentRepo,
    selectSkill,
    setSearchQuery,
    filteredSkills,
  }
})