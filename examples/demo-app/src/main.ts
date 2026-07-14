/**
 * Demo app UI: a key-value editor over tauri-plugin-icloud-kvs.
 * Doubles as the plugin's manual test rig — errors are surfaced
 * verbatim in the error bar rather than swallowed.
 */

import {
   accountStatus,
   getAll,
   remove,
   set,
   synchronize,
   type KvsValue,
} from 'tauri-plugin-icloud-kvs-api';

const statusBadge = document.querySelector<HTMLSpanElement>('#account-status')!,
      syncButton = document.querySelector<HTMLButtonElement>('#synchronize')!,
      syncResult = document.querySelector<HTMLSpanElement>('#sync-result')!,
      errorBar = document.querySelector<HTMLDivElement>('#error-bar')!,
      form = document.querySelector<HTMLFormElement>('#kv-form')!,
      keyInput = document.querySelector<HTMLInputElement>('#kv-key')!,
      valueInput = document.querySelector<HTMLTextAreaElement>('#kv-value')!,
      refreshButton = document.querySelector<HTMLButtonElement>('#refresh')!,
      tableBody = document.querySelector<HTMLTableSectionElement>('#kv-table tbody')!;

function showError(err: unknown): void {
   errorBar.textContent = String(err);
   errorBar.hidden = false;
}

function clearError(): void {
   errorBar.hidden = true;
}

/**
 * Parses the value textarea: JSON when possible, otherwise the raw text
 * as a string (convenience for quick manual tests).
 */
function parseValue(raw: string): KvsValue {
   try {
      return JSON.parse(raw) as KvsValue;
   } catch {
      return raw;
   }
}

async function updateAccountStatus(): Promise<void> {
   try {
      const status = await accountStatus();

      statusBadge.textContent = status;
      statusBadge.className = 'badge ' + (status === 'available' ? 'available' : 'no-account');
   } catch (err) {
      statusBadge.textContent = 'unknown';
      showError(err);
   }
}

async function refresh(): Promise<void> {
   try {
      const all = await getAll();

      tableBody.replaceChildren();

      for (const [key, value] of Object.entries(all)) {
         const row = document.createElement('tr'),
               keyCell = document.createElement('td'),
               valueCell = document.createElement('td'),
               actionCell = document.createElement('td'),
               deleteButton = document.createElement('button');

         keyCell.textContent = key;
         valueCell.textContent = JSON.stringify(value, null, 2);
         valueCell.className = 'value';
         deleteButton.textContent = 'Delete';
         deleteButton.addEventListener('click', async (event) => {
            event.stopPropagation();
            clearError();
            try {
               await remove(key);
               await refresh();
            } catch (err) {
               showError(err);
            }
         });
         row.addEventListener('click', () => {
            keyInput.value = key;
            valueInput.value = JSON.stringify(value, null, 2);
         });

         actionCell.appendChild(deleteButton);
         row.append(keyCell, valueCell, actionCell);
         tableBody.appendChild(row);
      }
   } catch (err) {
      showError(err);
   }
}

form.addEventListener('submit', async (event) => {
   event.preventDefault();
   clearError();
   try {
      await set(keyInput.value, parseValue(valueInput.value));
      await refresh();
   } catch (err) {
      showError(err);
   }
});

refreshButton.addEventListener('click', () => {
   clearError();
   void refresh();
});

syncButton.addEventListener('click', async () => {
   clearError();
   try {
      const flushed = await synchronize();

      syncResult.textContent = `synchronize() → ${String(flushed)}`;
   } catch (err) {
      showError(err);
   }
});

window.addEventListener('focus', () => void updateAccountStatus());

void updateAccountStatus();
void refresh();
