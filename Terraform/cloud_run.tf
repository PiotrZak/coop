resource "google_cloud_run_v2_service" "coop" {
  name     = var.service_name
  location = var.region
  project  = var.project_id

  template {
    containers {
      # First apply may use placeholder; CI overwrites with real image.
      image = "${var.gar_location}-docker.pkg.dev/${var.project_id}/${var.service_name}/${var.image_tag}"

      ports {
        container_port = 8080
      }

      env {
        name  = "RUST_LOG"
        value = "info"
      }

      resources {
        limits = {
          cpu    = "1"
          memory = "512Mi"
        }
      }
    }

    scaling {
      min_instance_count = 0
      max_instance_count = 3
    }
  }

  traffic {
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
    percent = 100
  }

  depends_on = [google_artifact_registry_repository.coop]
}

resource "google_cloud_run_v2_service_iam_member" "public_invoker" {
  count = var.allow_unauthenticated ? 1 : 0

  project  = var.project_id
  location = var.region
  name     = google_cloud_run_v2_service.coop.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}
