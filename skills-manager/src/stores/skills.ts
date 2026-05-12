import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface SkillMeta {
  id: string
  repo_id: string
  name: string
  description: string
  path: string
  local_path: string
  is_selected: boolean
}

export const useSkillsStore = defineStore('skills', () => {
  const skills = ref<SkillMeta[]>([])
  const currentRepoId = ref<string | null>(null)
  const selectedSkill = ref<SkillMeta | null>(null)
  const searchQuery = ref('')

  function setSkills(skillsList: SkillMeta[]) {
    skills.value = skillsList
  }

  function addSkills(skillsList: SkillMeta[]) {
    // Remove existing skills from the same repo, then add new ones
    const existingRepoIds = skillsList.map(s => s.repo_id)
    skills.value = skills.value.filter(s => !existingRepoIds.includes(s.repo_id))
    skills.value = [...skills.value, ...skillsList]
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

  function updateSkillIsSelected(skillId: string, isSelected: boolean) {
    const skill = skills.value.find(s => s.id === skillId)
    if (skill) {
      skill.is_selected = isSelected
    }
  }

  return {
    skills,
    currentRepoId,
    selectedSkill,
    searchQuery,
    setSkills,
    addSkills,
    setCurrentRepo,
    selectSkill,
    setSearchQuery,
    filteredSkills,
    updateSkillIsSelected,
  }
})