ALTER TABLE project_policies ADD COLUMN view_plugin_definition BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE project_policies ADD COLUMN create_plugin_definition BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE project_policies ADD COLUMN update_plugin_definition BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE project_policies ADD COLUMN delete_plugin_definition BOOLEAN NOT NULL DEFAULT false;
