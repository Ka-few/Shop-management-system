<template>
  <nav v-if="totalItems > 0" class="pagination" aria-label="Pagination">
    <span>{{ startItem }}-{{ endItem }} of {{ totalItems }}</span>
    <select :value="pageSize" @change="updatePageSize">
      <option v-for="size in pageSizes" :key="size" :value="size">{{ size }} / page</option>
    </select>
    <button type="button" :disabled="page <= 1" @click="$emit('update:page', page - 1)">Previous</button>
    <strong>Page {{ page }} of {{ totalPages }}</strong>
    <button type="button" :disabled="page >= totalPages" @click="$emit('update:page', page + 1)">Next</button>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  page: number
  pageSize: number
  totalItems: number
  pageSizes?: number[]
}>(), {
  pageSizes: () => [10, 25, 50, 100],
})

const emit = defineEmits<{
  'update:page': [value: number]
  'update:pageSize': [value: number]
}>()

const totalPages = computed(() => Math.max(1, Math.ceil(props.totalItems / props.pageSize)))
const startItem = computed(() => Math.min(props.totalItems, (props.page - 1) * props.pageSize + 1))
const endItem = computed(() => Math.min(props.totalItems, props.page * props.pageSize))

const updatePageSize = (event: Event) => {
  emit('update:pageSize', Number((event.target as HTMLSelectElement).value))
  emit('update:page', 1)
}
</script>

<style scoped>
.pagination {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 10px;
}

.pagination span,
.pagination strong {
  color: var(--color-muted);
  font-size: 0.9rem;
}

.pagination select {
  min-height: 36px;
  border: 1px solid #d7c58b;
  border-radius: 6px;
  background: var(--color-white);
  color: var(--color-black);
  padding: 0 8px;
}

.pagination button {
  min-height: 36px;
  border: 1px solid var(--color-black);
  border-radius: 6px;
  background: var(--color-black);
  color: var(--color-gold);
  padding: 0 12px;
  cursor: pointer;
  font-weight: 800;
}

.pagination button:hover:not(:disabled) {
  background: var(--color-gold);
  color: var(--color-black);
  border-color: var(--color-gold);
}

.pagination button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
