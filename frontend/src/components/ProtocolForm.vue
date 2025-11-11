<script setup lang="ts">
interface ProtocolFormModel {
  name: string;
  peptideName: string;
  notes: string;
  targetConcentration: string | number;
}

const props = defineProps<{
  form: ProtocolFormModel;
  saving: boolean;
}>();

const emit = defineEmits<{
  submit: [];
}>();

function handleSubmit() {
  emit("submit");
}
</script>

<template>
  <article class="panel">
    <div class="panel-header">
      <h2>New Protocol</h2>
    </div>
    <form class="form-stack" @submit.prevent="handleSubmit">
      <label>
        Name
        <input v-model="props.form.name" type="text" placeholder="Protocol name" />
      </label>
      <label>
        Peptide
        <input
          v-model="props.form.peptideName"
          type="text"
          placeholder="BPC-157, TB-500..."
        />
      </label>
      <label>
        Target concentration (mg/mL)
        <input
          v-model="props.form.targetConcentration"
          type="number"
          min="0"
          step="0.01"
        />
      </label>
      <label>
        Notes
        <textarea
          v-model="props.form.notes"
          rows="3"
          placeholder="Reconstitution details, supplier info..."
        />
      </label>
      <button class="primary" type="submit" :disabled="props.saving">
        {{ props.saving ? "Saving..." : "Save protocol" }}
      </button>
    </form>
  </article>
</template>
