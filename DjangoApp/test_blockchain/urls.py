from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
]