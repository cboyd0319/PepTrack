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
      <h2>➕ Add New Peptide Plan</h2>
    </div>
    <form class="form-stack" @submit.prevent="handleSubmit">
      <label>
        Plan Name
        <input v-model="props.form.name" type="text" placeholder="e.g., Morning Routine, Recovery Plan" />
      </label>
      <label>
        Which Peptide?
        <input
          v-model="props.form.peptideName"
          type="text"
          placeholder="e.g., BPC-157, TB-500"
        />
      </label>
      <label>
        Strength (optional)
        <input
          v-model="props.form.targetConcentration"
          type="number"
          min="0"
          step="0.01"
          placeholder="mg/mL"
        />
      </label>
      <label>
        Notes (optional)
        <textarea
          v-model="props.form.notes"
          rows="3"
          placeholder="Where you bought it, how to mix it, storage instructions, etc."
        />
      </label>
      <button class="primary" type="submit" :disabled="props.saving">
        {{ props.saving ? "Saving..." : "💾 Save Plan" }}
      </button>
    </form>
  </article>
</template>
